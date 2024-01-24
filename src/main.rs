mod toots;
mod server;
mod prelude;

use dotenv::dotenv;
use std::env;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let account_url = env::var("ACCOUNT_URL").expect("ACCOUNT_URL is not set");

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting up");

    let toots: toots::Toots = Arc::new(Mutex::new(HashMap::new()));

    tokio::select! {
        err = toots::list(account_url, toots) => {
            error!("get_toots fails, error: {:?}", err);
        },
    }
}


