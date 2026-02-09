use axum::{Extension, Router, routing::get};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::net::TcpListener;
use tracing::{Level, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new().connect(&url).await?;
    println!("connected to the database");

    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let app = Router::new().route("/", get(root)).layer(Extension(pool));

    let listener = TcpListener::bind("0.0.0.0:6969").await.unwrap();
    info!("Server is running on http://0.0.0.0:6969");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn root() -> &'static str {
    "Hello there"
}
