pub mod state;
pub mod routes;
pub mod utils;
pub mod resources;
pub mod tools;

use actix_web::{web, App};
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};

use state::AppState;
use utils::{Client, Config};

pub fn build_app(
    app_state: web::Data<AppState>,
) -> App<impl ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse<impl MessageBody>, Error = actix_web::Error, InitError = ()>> {
    App::new()
        .app_data(app_state)
        .configure(routes::mcp::configure)
}

pub fn default_state() -> web::Data<AppState> {
    web::Data::new(AppState {
        client: Client::new(),
        config: Config::default(),
    })
}