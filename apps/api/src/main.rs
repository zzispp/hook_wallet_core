use auto_allocator;

use core_tracing::CoreTracing;
use rocket::tokio::sync::Mutex;
use rocket::{Build, Rocket, routes};
use settings::Settings;
use std::str::FromStr;
use tracing::{info, warn};

mod chain;
mod model;
mod params;
mod responders;
mod system;

use crate::chain::client::ChainClient;
use model::APIService;
use settings_chain::ChainProviders;

async fn rocket_api(settings: Settings) -> Rocket<Build> {
    let providers = ChainProviders::from_settings(&settings, "api");
    let chain_client = ChainClient::new(providers);

    let figment = rocket::Config::figment()
        .merge(("address", settings.server.host.clone()))
        .merge(("port", settings.server.port))
        .merge(("cli_colors", false));

    rocket::custom(figment)
        .manage(Mutex::new(chain_client))
        .mount(
            "/",
            routes![system::status::get_status, system::status::get_health],
        )
        .mount(
            "/v1",
            routes![
                chain::balance::get_balances_coin,
                chain::balance::get_balances_assets,
                chain::balance::get_balances_staking,
            ],
        )
}

#[tokio::main]
async fn main() {
    let info = auto_allocator::get_allocator_info();
    let settings = Settings::new().unwrap();

    let _tracing = CoreTracing::init(&settings, "api");

    warn!("currently used allocator type {:?}", info.allocator_type);
    warn!("detailed reason for allocator selection {:?}", info.reason);
    warn!(
        "system hardware and environment information: {:?}",
        info.system_info
    );

    let service = std::env::args().nth(1).unwrap_or_default();
    let service = APIService::from_str(service.as_str())
        .ok()
        .unwrap_or(APIService::Api);

    info!("api start service: {}", service.as_ref());

    match service {
        APIService::Api => {
            let rocket_api = rocket_api(settings.clone()).await;
            rocket_api.launch().await.expect("Failed to launch Rocket");
        }
        APIService::WebsocketPrices => todo!(),
    }
}
