use {
    axum::{
        body::Bytes,
        http::Request,
        middleware::{self},
        response::Response,
        routing::{get, post},
        Router,
    },
    axum_extra::routing::SpaRouter,
    lshort::metrics::{setup_metrics_recorder, track_metrics},
    lshort::routes::{global_404, health_check, new_link, redirect},
    sqlx::postgres::PgPoolOptions,
    std::{future::ready, net::SocketAddr, time::Duration},
    tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer},
    tracing::Span,
    tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "lshort=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let recorder_handle = setup_metrics_recorder();

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect("postgres://postgres:password@localhost:5432/lshort")
        .await
        .expect("Cannot connect to database");

    let web = SpaRouter::new("/", "web/dist"); // serving the frontend react app

    let app = Router::with_state(pool)
        .route("/health_check", get(health_check))
        .route("/n", post(new_link))
        .route("/r/:id", get(redirect))
        .merge(web)
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
