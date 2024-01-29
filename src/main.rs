mod error;
mod prelude;
mod server;
mod toots;

use dotenv::dotenv;
use prelude::*;
use std::env;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
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

    info!("starting up");

    let toots: toots::Toots = Arc::new(Mutex::new(HashMap::new()));
    let toots_clone = toots.clone();

    tokio::select! {
        err = toots::list(account_url, toots_clone) => {
            error!("get_toots fails, error: {:?}", err);
        },
        err = server::serve_toots(toots) => {
            error!("serve_toots fails, error: {:?}", err);
        },
    }
}
