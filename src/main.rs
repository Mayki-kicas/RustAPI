use axum::{
    routing::get,
    Json,
    Router,
    response::IntoResponse,
};
use serde::Serialize;
use std::net::SocketAddr;
use dotenv::dotenv;
use std::env;
use deadpool_postgres::{Pool, Manager};
use tokio_postgres::{NoTls, Error as PgError};
use std::str::FromStr;
use std::fmt;

#[derive(Serialize)]
struct Book {
    id: i32,
    title: String,
    author: String,
    publication_date: String,
}

#[derive(Debug)]
enum AppError {
    PoolError(deadpool::managed::errors::PoolError<PgError>),
    PgError(PgError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AppError {}

impl From<deadpool::managed::errors::PoolError<PgError>> for AppError {
    fn from(err: deadpool::managed::errors::PoolError<PgError>) -> Self {
        AppError::PoolError(err)
    }
}

impl From<PgError> for AppError {
    fn from(err: PgError) -> Self {
        AppError::PgError(err)
    }
}

async fn get_books_handler(pool: Pool) -> Result<impl IntoResponse, AppError> {
    let client = pool.get().await.map_err(AppError::from)?;
    let rows = client
        .query(
            "SELECT id, title, author, publication_date FROM books",
            &[],
        )
        .await
        .map_err(AppError::from)?;

    let books: Vec<Book> = rows
        .iter()
        .map(|row| Book {
            id: row.get(0),
            title: row.get(1),
            author: row.get(2),
            publication_date: row.get(3),
        })
        .collect();

    Ok(Json(books))
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let config = tokio_postgres::Config::from_str(&database_url).unwrap();
    let manager = deadpool_postgres::Manager::new(config, NoTls);

    let pool = deadpool_postgres::Pool::new(manager, 16);

    let app = Router::new().route("/books", get({
        let pool = pool.clone();
        move || get_books_handler(pool.clone())
    }));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
