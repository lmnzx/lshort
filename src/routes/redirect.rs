use {
    axum::{
        extract::{Path, State},
        response::{IntoResponse, Redirect},
    },
    sqlx::postgres::PgPool,
};

pub async fn redirect(State(pool): State<PgPool>, Path(params): Path<String>) -> impl IntoResponse {
    let url = sqlx::query!(
        r#"
            SELECT url FROM links WHERE key=$1
        "#,
        params
    )
    .fetch_one(&pool)
    .await
    .unwrap()
    .url;

    println!("{:#?}", url);

    Redirect::to(&url).into_response()
}
