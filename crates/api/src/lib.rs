#![recursion_limit = "512"]
pub mod error;
pub mod handlers;
pub mod pagination;
pub mod state;

use axum::{
    http::{HeaderValue, Method},
    response::IntoResponse,
    routing::{get, post},
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
        .route(
            "/trees/:tree_id/research-tasks",
            get(handlers::research_tasks::list_tasks).post(handlers::research_tasks::create_task),
        )
        .route(
            "/trees/:tree_id/research-tasks/:task_id",
            get(handlers::research_tasks::get_task)
                .patch(handlers::research_tasks::update_task)
                .delete(handlers::research_tasks::delete_task),
        )
        .route(
            "/trees/:tree_id/research-opportunities/:opportunity_id/tasks",
            post(handlers::research_tasks::create_task_from_opportunity),
        )
        .route(
            "/trees/:tree_id/research-tasks/:task_id/outcome",
            post(handlers::research_outcomes::create_outcome),
        )
        .route(
            "/trees/:tree_id/research-outcomes",
            get(handlers::research_outcomes::list_outcomes),
        )
        .route(
            "/trees/:tree_id/research-outcomes/:outcome_id",
            get(handlers::research_outcomes::get_outcome)
                .patch(handlers::research_outcomes::update_outcome)
                .delete(handlers::research_outcomes::delete_outcome),
        )
        .route(
            "/trees/:tree_id/research/summary",
            get(handlers::research_summary::get_research_summary),
        )
        .route(
            "/trees/:tree_id/sources",
            get(handlers::sources::list_sources).post(handlers::sources::create_source),
        )
        .route(
            "/trees/:tree_id/sources/:source_id",
            get(handlers::sources::get_source)
                .patch(handlers::sources::update_source)
                .delete(handlers::sources::delete_source),
        )
        .route(
            "/trees/:tree_id/sources/:source_id/citations",
            get(handlers::citations::list_citations).post(handlers::citations::create_citation),
        )
        .route(
            "/trees/:tree_id/citations/:citation_id",
            get(handlers::citations::get_citation)
                .patch(handlers::citations::update_citation)
                .delete(handlers::citations::delete_citation),
        )
        .route(
            "/trees/:tree_id/evidence",
            get(handlers::evidence::list_evidence).post(handlers::evidence::create_evidence),
        )
        .route(
            "/trees/:tree_id/evidence/:evidence_id",
            get(handlers::evidence::get_evidence)
                .patch(handlers::evidence::update_evidence)
                .delete(handlers::evidence::delete_evidence),
        )
        .route(
            "/trees/:tree_id/research-outcomes/:outcome_id/evidence",
            get(handlers::evidence::list_outcome_evidence),
        )
        .route(
            "/trees/:tree_id/research-outcomes/:outcome_id/evidence/:evidence_id",
            post(handlers::evidence::attach_evidence).delete(handlers::evidence::detach_evidence),
        )
        .route(
            "/trees/:tree_id/research-outcomes/:outcome_id/followup-actions",
            get(handlers::followup_actions::list_outcome_actions)
                .post(handlers::followup_actions::create_action),
        )
        .route(
            "/trees/:tree_id/research-followup-actions",
            get(handlers::followup_actions::list_actions),
        )
        .route(
            "/trees/:tree_id/research-followup-actions/:action_id",
            get(handlers::followup_actions::get_action)
                .patch(handlers::followup_actions::update_action)
                .delete(handlers::followup_actions::delete_action),
        )
        .route(
            "/trees/:tree_id/research-tasks/:task_id/followup-actions",
            get(handlers::followup_actions::list_task_actions),
        )
        .route(
            "/trees/:tree_id/research-tasks/:task_id/case-summary",
            get(handlers::case_summary::get_case_summary),
        )
        .route(
            "/trees/:tree_id/research/plan",
            get(handlers::planning::get_research_plan),
        )
        .route(
            "/trees/:tree_id/research-sessions",
            get(handlers::research_sessions::list_sessions)
                .post(handlers::research_sessions::create_session),
        )
        .route(
            "/trees/:tree_id/research-sessions/:session_id",
            get(handlers::research_sessions::get_session)
                .patch(handlers::research_sessions::update_session)
                .delete(handlers::research_sessions::delete_session),
        )
        .route(
            "/trees/:tree_id/research-sessions/:session_id/tasks",
            get(handlers::research_sessions::list_session_tasks),
        )
        .route(
            "/trees/:tree_id/research-tasks/:task_id/session",
            post(handlers::research_sessions::assign_task_to_session)
                .delete(handlers::research_sessions::remove_task_from_session),
        )
        // Generic session routes for spec compatibility (tree_id in body or path-less)
        .route(
            "/research-sessions",
            get(handlers::research_sessions::list_sessions_generic)
                .post(handlers::research_sessions::create_session_generic),
        )
        .route(
            "/research-sessions/:session_id",
            get(handlers::research_sessions::get_session_generic)
                .patch(handlers::research_sessions::patch_session_generic)
                .delete(handlers::research_sessions::delete_session_generic),
        )
        .route("/openapi.json", get(handlers::openapi::get_openapi))
        .route("/docs", get(handlers::openapi::get_docs));

    let mut app = Router::new()
        .route("/health", get(handlers::health::health))
        .route("/ready", get(handlers::health::health))
        .nest("/api/v1", api)
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http())
        .layer(cors_layer());

    // Serve web/dist if exists (for single-origin deployment)
    if std::path::Path::new("web/dist/index.html").exists() {
        let svc = tower_http::services::ServeDir::new("web/dist").not_found_service(
            tower::service_fn(|_req: axum::http::Request<axum::body::Body>| async {
                let index = tokio::fs::read_to_string("web/dist/index.html")
                    .await
                    .unwrap_or_default();
                Ok::<_, std::convert::Infallible>(axum::response::Html(index).into_response())
            }),
        );
        app = app.fallback_service(svc);
    }
    app
}

fn cors_layer() -> CorsLayer {
    if let Ok(origin) = std::env::var("NEOGENEALOGY_CORS_ORIGIN") {
        if origin == "*" {
            CorsLayer::permissive()
        } else {
            CorsLayer::new()
                .allow_origin(origin.parse::<HeaderValue>().unwrap())
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PATCH,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
        }
    } else {
        CorsLayer::new()
            .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
            .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
            .allow_origin("http://127.0.0.1:3000".parse::<HeaderValue>().unwrap())
            .allow_origin("http://127.0.0.1:5173".parse::<HeaderValue>().unwrap())
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([axum::http::header::CONTENT_TYPE])
    }
}
