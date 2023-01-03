use {
    axum::{
        extract::{Json, State},
        http::StatusCode,
        response::Json as ResponseJson,
    },
    chrono::Utc,
    rand::distributions::{Alphanumeric, DistString},
    serde_json::{json, Value},
    sqlx::postgres::PgPool,
    uuid::Uuid,
};

#[derive(serde::Deserialize)]
pub struct Link {
    value: String,
}

pub async fn new_link(
    State(pool): State<PgPool>,
    Json(link): Json<Link>,
) -> (StatusCode, ResponseJson<Value>) {
    let id = Uuid::new_v4();

    let key = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);

    let endpoint = std::env::var("APP_ENDPOINT").unwrap_or_else(|_| "localhost:3000/".into());

    match sqlx::query!(
        r#"
            INSERT INTO links (id, url, key, created_at)
            VALUES ($1, $2, $3, $4)
        "#,
        id,
        link.value,
        key,
        Utc::now()
    )
    .execute(&pool)
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            ResponseJson(json!({ "data": format!("{}r/{}", endpoint, key) })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ResponseJson(json!({"error": "oops we messed up..."})),
        ),
    }
}
