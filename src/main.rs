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
    hyper::http::{header, HeaderValue, Method},
    lshort::config::{get_config, Environment},
    lshort::metrics::{setup_metrics_recorder, track_metrics},
    lshort::routes::{global_404, health_check, new_link, redirect},
    sqlx::postgres::PgPoolOptions,
    std::{future::ready, time::Duration},
    tower_http::{classify::ServerErrorsFailureClass, cors::CorsLayer, trace::TraceLayer},
    tracing::Span,
    tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt},
};

// TODO Add link validation

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "lshort=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let recorder_handle = setup_metrics_recorder();

    let config = get_config().expect("Failed to read configuration");

    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(10))
        .connect_lazy_with(config.database.get_db());

    let web = SpaRouter::new("/", "web/dist"); // serving the frontend react app

    // Only needed for dev env as axum will server the build app from same origin
    let env: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    let cors;

    match env {
        Environment::Local => {
            cors = CorsLayer::new()
                .allow_credentials(true)
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([header::CONTENT_TYPE]);
        }

        Environment::Production => cors = CorsLayer::new(),
    }

    let app = Router::new()
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
        )
        .with_state(pool)
        .layer(cors);

    let addr = config.application.get_address();

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
