use axum::{
    Extension, Json, Router,
    extract::Path,
    http::StatusCode,
    routing::{get, post},
};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::env;
use tokio::net::TcpListener;
use tracing::{Level, info};
use tracing_subscriber;

#[derive(Serialize, Deserialize)]
struct CreateUser {
    username: String,
    email: String,
}

#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
    email: String,
}

#[derive(Serialize)]
struct Message {
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Post {
    id: i32,
    user_id: Option<i32>,
    title: String,
    body: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CreatePost {
    title: String,
    body: String,
    user_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UpdatePost {
    title: String,
    body: String,
    user_id: Option<i32>,
}

async fn create_user(
    Extension(pool): Extension<Pool<Postgres>>,
    Json(new_user): Json<CreateUser>,
) -> Result<Json<User>, StatusCode> {
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id, username, email",
        new_user.username,
        new_user.email
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user))
}

async fn delete_post(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, StatusCode> {
    let result = sqlx::query!("DELETE FROM posts WHERE id = $1", id)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => Ok(Json(json!({
            "message": "Post deleted successfully"
        }))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn update_post(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
    Json(updated_post): Json<UpdatePost>,
) -> Result<Json<Post>, StatusCode> {
    let post = sqlx::query_as!(
        Post,
        "UPDATE posts SET title = $1, body = $2, user_id = $3 WHERE id = $4 RETURNING id, user_id, title, body",
        updated_post.title,
        updated_post.body,
        updated_post.user_id,
        id
    )
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(post))
}

async fn create_post(
    Extension(pool): Extension<Pool<Postgres>>,
    Json(new_post): Json<CreatePost>,
) -> Result<Json<Post>, StatusCode> {
    let post = sqlx::query_as!(
        Post,
        "INSERT INTO posts (user_id, title, body) values ($1, $2, $3) RETURNING id, title, body, user_id",
        new_post.user_id,
        new_post.title,
        new_post.body
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(post))
}

async fn get_posts(
    Extension(pool): Extension<Pool<Postgres>>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<Post>>, StatusCode> {
    let posts = sqlx::query_as!(
        Post,
        "SELECT id, user_id, title, body FROM posts WHERE id = $1",
        id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(posts))
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new().connect(&url).await?;
    println!("connected to the database");

    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let app = Router::new()
        .route("/users", post(create_user))
        .route("/posts", post(create_post))
        .route(
            "/posts/{id}",
            get(get_posts).put(update_post).delete(delete_post),
        )
        .layer(Extension(pool));

    let listener = TcpListener::bind("0.0.0.0:6969").await.unwrap();
    info!("Server is running on http://0.0.0.0:6969");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn root() -> &'static str {
    "Hello there"
}
