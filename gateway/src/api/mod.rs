pub mod models;
pub(crate) mod response;
mod routes;
pub(crate) mod trace;

use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::{get, post};

use crate::auth;
use crate::config::GatewayConfig;
use crate::services::GatewayService;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<GatewayConfig>,
    pub service: Arc<dyn GatewayService>,
}

impl AppState {
    pub fn new(config: GatewayConfig, service: Arc<dyn GatewayService>) -> Self {
        Self {
            config: Arc::new(config),
            service,
        }
    }
}

pub fn router(config: GatewayConfig, service: Arc<dyn GatewayService>) -> Router {
    let state = AppState::new(config, service);

    let business_routes = Router::new()
        .route("/payments", post(routes::create_payment))
        .route("/payments/{merchant_order_id}", get(routes::query_payment))
        .route("/refunds", post(routes::create_refund))
        .route("/refunds/{merchant_refund_id}", get(routes::query_refund))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_api_key,
        ));

    let public_routes = Router::new()
        .route("/health/live", get(routes::liveness))
        .route("/health/ready", get(routes::readiness))
        .route(
            "/webhooks/{provider}/payments",
            post(routes::receive_payment_webhook),
        )
        .route(
            "/webhooks/{provider}/refunds",
            post(routes::receive_refund_webhook),
        );

    Router::new()
        .nest("/v1", public_routes.merge(business_routes))
        .with_state(state)
}
