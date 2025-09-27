use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use axum::{Extension, Json, response::IntoResponse};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;

use crate::{
    config::ApiConfig,
    data::{ApiData, Candidate},
    doc,
    error::ApiErr,
};

pub mod routes;

#[derive(Clone)]
struct ApiCtx {
    pub config: Arc<ApiConfig>,
    pub data: Arc<ApiData>,
}

pub async fn run_server(config: ApiConfig) {
    let data = ApiData::init(&config);

    let ctx = ApiCtx {
        config: Arc::new(config.clone()),
        data: Arc::new(data),
    };

    // init cors
    let cors = CorsLayer::new()
        .allow_headers(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .expose_headers(tower_http::cors::Any)
        .allow_origin(tower_http::cors::Any);

    let mut router: axum::Router = axum::Router::new()
        .route("/", axum::routing::get(root_route))
        // vote
        .merge(routes::vote::router());
    router = router
        // swagger
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/swagger").url("/openapi", doc::ApiDoc::openapi()),
        );

    // // Common middleware for all routes
    router = router.layer(
        tower::ServiceBuilder::new()
            .layer(cors)
            .layer(axum::Extension(ctx))
            .into_inner(),
    );

    let ip = Ipv4Addr::from_str(&config.ip).expect("Fail parse IP Address from config");
    let port = config.port;

    let addr = SocketAddr::new(IpAddr::V4(ip), port);

    log::info!("listen on: {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap()
}

async fn root_route(ctx: Extension<ApiCtx>) -> String {
    let v = format!("[OK]");
    v
}

async fn get_vote_candidates(ctx: Extension<ApiCtx>) -> Result<impl IntoResponse, ApiErr> {
    Ok(Json(vec!["stuff"]))
}
