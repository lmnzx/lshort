use {
    axum::{
        body::Bytes,
        extract::{Extension, Json},
        http::{Request, StatusCode},
        middleware::{self},
        response::{IntoResponse, Response},
        routing::{get, post},
        Router,
    },
    lshort::metrics::{setup_metrics_recorder, track_metrics},
    rand::distributions::{Alphanumeric, DistString},
    redis::{Client, Commands},
    std::sync::Arc,
    std::{future::ready, net::SocketAddr, time::Duration},
    tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer},
    tracing::Span,
    tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt},
};

#[derive(serde::Deserialize)]
struct Link {
    value: String,
}

async fn handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn new_link(
    Extension(client): Extension<Arc<Client>>,
    Json(link): Json<Link>,
) -> impl IntoResponse {
    let mut con = client.get_connection().expect("Failed to connect to redis");
    println!("{}", link.value);
    let s = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);
    let _save_to_redis: () = con.set(link.value, &s).expect("Failed to set key");
    (StatusCode::OK, format!("localhost:3000/r/{}", s))
}

pub async fn global_404() -> impl IntoResponse {
    let message = "
     __      __                              __ __       __   __ __      
   /'__`\\  /'__`\\                           /\\ \\\\ \\    /'__`\\/\\ \\\\ \\     
  /\\ \\/\\ \\/\\ \\/\\ \\  _____     ____          \\ \\ \\\\ \\  /\\ \\/\\ \\ \\ \\\\ \\    
  \\ \\ \\ \\ \\ \\ \\ \\ \\/\\ '__`\\  /',__\\          \\ \\ \\\\ \\_\\ \\ \\ \\ \\ \\ \\\\ \\_  
   \\ \\ \\_\\ \\ \\ \\_\\ \\ \\ \\L\\ \\/\\__, `\\__  __  __\\ \\__ ,__\\ \\ \\_\\ \\ \\__ ,__\\
    \\ \\____/\\ \\____/\\ \\ ,__/\\/\\____/\\_\\/\\_\\/\\_\\\\/_/\\_\\_/\\ \\____/\\/_/\\_\\_/
     \\/___/  \\/___/  \\ \\ \\/  \\/___/\\/_/\\/_/\\/_/   \\/_/   \\/___/    \\/_/  
                      \\ \\_\\                                              
                       \\/_/                                              
  ";
    (StatusCode::NOT_FOUND, message)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "lshort=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let recorder_handle = setup_metrics_recorder();

    let redis_client =
        redis::Client::open("redis://127.0.0.1:6379").expect("Failed to connect to redis");

    let redis_client = Arc::new(redis_client);

    let app = Router::new()
        .route("/", get(handler))
        .route("/new", post(new_link))
        .layer(Extension(redis_client))
        .fallback(global_404)
        .layer(TraceLayer::new_for_http())
        .route("/metrics", get(move || ready(recorder_handle.render())))
        .route_layer(middleware::from_fn(track_metrics))
        .layer(
            TraceLayer::new_for_http()
                .on_request(|request: &Request<_>, _span: &Span| {
                    tracing::debug!("started {} {}", request.method(), request.uri().path())
                })
                .on_response(|_response: &Response, latency: Duration, _span: &Span| {
                    tracing::debug!("response generated in {:?}", latency)
                })
                .on_body_chunk(|chunk: &Bytes, _latency: Duration, _span: &Span| {
                    tracing::debug!("sending {} bytes", chunk.len())
                })
                .on_failure(
                    |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        tracing::error!("something went wrong {:#?}", error)
                    },
                ),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
