use axum::{Router, middleware};
use crate::AppState;

mod auth;
mod users;
mod products;
mod transactions;
mod tickets;
mod reports;
mod settings;
mod health;
mod payment_methods;
mod categories;
mod variants;
mod investors;

use crate::middleware::auth::require_auth;

pub fn build_router(state: AppState) -> Router {
    // Routes that require authentication
    let protected = Router::new()
        .nest("/users",                          users::router())
        .nest("/investors",                      investors::router())
        .nest("/products",                       products::router())
        .nest("/products/:product_id/variants",  variants::router())
        .nest("/transactions",                   transactions::router())
        .nest("/tickets",                        tickets::router())
        .nest("/reports",                        reports::router())
        .nest("/settings",                       settings::router())
        .nest("/payment-methods",                payment_methods::router())
        .nest("/categories",                     categories::router())
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    // Combine public + protected
    Router::new()
        .nest("/auth",   auth::router())
        .nest("/health", health::router())
        .merge(protected)
        .with_state(state)
}
