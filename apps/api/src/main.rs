use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use core_tracing::CoreTracing;
use settings::Settings;
use std::io::Result;
use std::str::FromStr;
use tracing::info;

mod model;
mod system;

use model::APIService;
use crate::system::status::server_status;

#[tokio::main]
async fn main() -> Result<()> {
    // 加载配置
    let settings = Settings::new().unwrap();

    // 初始化 tracing
    let _tracing = CoreTracing::init(&settings, "api");
    let service = std::env::args().nth(1).unwrap_or_default();
    let service = APIService::from_str(service.as_str())
        .ok()
        .unwrap_or(APIService::Api);

    info!("api start service: {}", service.as_ref());

    match service {
        APIService::Api => actix_web_run(&settings).await?.await,
        APIService::WebsocketPrices => todo!(),
    }
}

async fn actix_web_run(settings: &Settings) -> Result<Server> {
    Ok(HttpServer::new(move || App::new().service(web::scope("/api").service(web::scope("/system").service(server_status))))
        .bind((settings.server.host.clone(), settings.server.port))?
        .run())
}
