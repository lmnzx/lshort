use {
    axum::{
        extract::{Json, State},
        http::StatusCode,
        response::IntoResponse,
    },
    chrono::Utc,
    rand::distributions::{Alphanumeric, DistString},
    sqlx::postgres::PgPool,
    uuid::Uuid,
};

#[derive(serde::Deserialize)]
pub struct Link {
    value: String,
}

pub async fn new_link(State(pool): State<PgPool>, Json(link): Json<Link>) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let key = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);
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
        Ok(_) => (StatusCode::OK, format!("localhost:3000/r/{}", key)),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "oops we messed up".to_owned(),
        ),
    }
}
