pub mod error;
pub mod handlers;
pub mod pagination;
pub mod state;

use axum::{
    http::{HeaderValue, Method},
    routing::get,
    Router,
};
use state::AppState;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn create_router(state: AppState) -> Router {
    let api = Router::new()
        .route("/trees", get(handlers::trees::list_trees))
        .route("/trees/:tree_id", get(handlers::trees::get_tree))
        .route(
            "/trees/:tree_id/persons",
            get(handlers::persons::list_persons),
        )
        .route(
            "/trees/:tree_id/persons/:person_id",
            get(handlers::persons::get_person),
        )
        .route(
            "/trees/:tree_id/families",
            get(handlers::families::list_families),
        )
        .route(
            "/trees/:tree_id/families/:family_id",
            get(handlers::families::get_family),
        )
        .route(
            "/trees/:tree_id/findings",
            get(handlers::findings::list_findings),
        )
        .route(
            "/trees/:tree_id/research-opportunities",
            get(handlers::opportunities::list_opportunities),
        )
        .route(
            "/trees/:tree_id/research-opportunities/top",
            get(handlers::opportunities::top_opportunities),
        )
        .route(
            "/trees/:tree_id/branches",
            get(handlers::branches::list_branches),
        )
        .route(
            "/trees/:tree_id/source-coverage",
            get(handlers::coverage::get_coverage),
        )
        .route(
            "/trees/:tree_id/analysis-runs",
            get(handlers::runs::list_runs),
        )
        .route(
            "/trees/:tree_id/analysis-runs/:run_id",
            get(handlers::runs::get_run),
        )
        .route("/openapi.json", get(handlers::openapi::get_openapi))
        .route("/docs", get(handlers::openapi::get_docs));

    Router::new()
        .route("/health", get(handlers::health::health))
        .route("/ready", get(handlers::health::health))
        .nest("/api/v1", api)
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer())
}

fn cors_layer() -> CorsLayer {
    if let Ok(origin) = std::env::var("NEOGENEALOGY_CORS_ORIGIN") {
        if origin == "*" {
            CorsLayer::permissive()
        } else {
            CorsLayer::new()
                .allow_origin(origin.parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET])
        }
    } else {
        CorsLayer::new()
            .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
            .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET])
    }
}
