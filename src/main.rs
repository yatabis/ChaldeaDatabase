use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use actix_web::middleware::Logger;

use chaldea_database::kizuna;

#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Chaldea Database");
    let port = std::env::var("PORT").unwrap();
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(ping)
            .service(web::scope("/api")
                .service(web::scope("/kizuna")
                    .service(kizuna::get_tables)))
    })
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .await
}
