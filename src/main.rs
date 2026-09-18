use actix_web::HttpServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = ariadne::default_state();

    HttpServer::new(move || ariadne::build_app(app_state.clone()))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}