use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::SqlitePool;

#[tokio::main]
async fn main() {
    let pool = SqlitePool::connect("sqlite:snip.db?mode=rwc")
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();

    let app = Router::new()
        .route("/{key}", get(get_link))
        .route("/create", post(create_link))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct CreateLinkRequest {
    original: String,
    slug: String,
}

async fn create_link(
    State(pool): State<SqlitePool>,
    Json(req): Json<CreateLinkRequest>,
) -> Result<StatusCode, StatusCode> {
    if req.slug.is_empty()
        || req.original.is_empty()
        || !req.slug.chars().all(|c| c.is_alphanumeric() || c == '-')
        || req.original.len() > 2048
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    sqlx::query!(
        "INSERT INTO links (original, slug) VALUES (?, ?)",
        req.original,
        req.slug,
    )
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::CONFLICT)?;

    Ok(StatusCode::CREATED)
}

struct LinkDto {
    original: String,
}

async fn get_link(
    State(pool): State<SqlitePool>,
    Path(slug): Path<String>,
) -> Result<Redirect, StatusCode> {
    let link = sqlx::query_as!(LinkDto, "SELECT original FROM links WHERE slug = ?", slug)
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let url = if link.original.starts_with("http") {
        link.original
    } else {
        format!("https://{}", link.original)
    };

    Ok(Redirect::temporary(&url))
}
