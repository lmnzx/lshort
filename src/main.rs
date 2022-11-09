use {
    axum::{
        body::Bytes,
        extract::{Json, Path, State},
        http::{Request, StatusCode},
        middleware::{self},
        response::{IntoResponse, Response},
        routing::{get, post},
        Router,
    },
    lshort::metrics::{setup_metrics_recorder, track_metrics},
    rand::distributions::{Alphanumeric, DistString},
    rocksdb::{Options, DB},
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

async fn new_link(State(db): State<DB>, Json(link): Json<Link>) -> impl IntoResponse {
    println!("{}", link.value);
    let s = Alphanumeric.sample_string(&mut rand::thread_rng(), 6);
    match db.put(s.as_bytes(), link.value.as_bytes()) {
        Ok(_) => return (StatusCode::OK, format!("localhost:3000/r/{}", s)),
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "oops we messed up".to_owned(),
            )
        }
    };
}

async fn redirect(State(db): State<DB>, Path(params): Path<String>) -> impl IntoResponse {
    let url = match db.get(params.as_bytes()) {
        Ok(Some(u)) => String::from_utf8(u).unwrap(),
        Ok(None) => "Not Found".to_owned(),
        Err(e) => format!("{}", e),
    };
    (StatusCode::OK, url)
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

    let db = DB::open_default("./database").unwrap();

    let app = Router::with_state(db)
        .route("/", get(handler))
        .route("/n", post(new_link))
        .route("/r/:id", get(redirect))
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
