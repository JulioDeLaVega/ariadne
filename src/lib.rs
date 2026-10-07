pub mod routes;
pub mod utils;
pub mod resources;
pub mod tools;

use actix_web::{web, App};
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::middleware::from_fn;

use utils::rate_limit::{rate_limit, RateLimiter};
use utils::{Client, Config, AppState};

pub fn build_app(
    app_state: web::Data<AppState>,
    limiter: web::Data<RateLimiter>,
) -> App<impl ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse<impl MessageBody>, Error = actix_web::Error, InitError = ()>> {
    App::new()
        .app_data(app_state)
        .app_data(limiter)
        .configure(routes::mcp::configure)
        .wrap(from_fn(rate_limit))
}

pub fn default_state() -> web::Data<AppState> {
    web::Data::new(AppState {
        client: Client::new(),
        config: Config::default(),
    })
}

pub fn default_limiter() -> web::Data<RateLimiter> {
    web::Data::new(RateLimiter::new(20, 5.0)) // burst 20, 5 req/s per IP
}