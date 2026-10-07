use std::time::Duration;
use actix_web::{HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = ariadne::default_state();
    let limiter = ariadne::default_limiter(); // burst of 20, sustained 5 req/s per IP

    // background cleanup of idle IPs
    let cleaner = limiter.clone();
    actix_web::rt::spawn(async move {
        let mut tick = actix_web::rt::time::interval(Duration::from_secs(60));
        loop {
            tick.tick().await;
            cleaner.evict_idle(Duration::from_secs(300));
        }
    });

    HttpServer::new(move || ariadne::build_app(app_state.clone(), limiter.clone()))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}