use crate::prelude::*;
use chrono::{DateTime, Utc};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

pub type Toots = Arc<Mutex<HashMap<String, Toot>>>;

#[derive(serde::Deserialize, Clone)]
pub struct TootIn {
    id: String,
    content: String,
    created_at: DateTime<Utc>,
}

pub struct Toot {
    id: String,
    content: String,
    created_at: DateTime<Utc>,
    served: bool,
}

impl Toot {
    pub fn try_from_toot_in(toot: TootIn) -> Result<Self> {
        let content = html_escape::decode_html_entities(&toot.content).to_string();
        let content = html2text::from_read(content.as_bytes(), 80).to_string();
        let content = content.split("\"").collect::<Vec<_>>();
        if content.len() < 2 {
            bail!("failed to parse toot content between \"");
        }
        let content = content[1].to_string();

        Ok(Self {
            id: toot.id,
            content: content.clone(),
            created_at: toot.created_at,
            served: false,
        })
    }

    pub fn content(&self) -> String {
        self.content.clone()
    }
}

pub async fn list(account_url: String, toots: Toots) -> Result<()> {
    let client = reqwest::Client::new();

    loop {
        sleep(Duration::from_secs(3)).await;

        info!("fetching toots from {account_url}");

        let resp = client
            .get(&format!("{account_url}/statuses?exclude_replies=true"))
            .send()
            .await?
            .json::<Vec<TootIn>>()
            .await?
            .into_iter()
            .filter(|toot| toot.created_at > Utc::now() - chrono::Duration::days(5))
            .filter(|toot| toot.content.len() > 10)
            .collect::<Vec<_>>();

        let Ok(mut toots) = toots.lock() else {
            error!("failed to lock toots");
            continue;
        };
        for toot in resp {
            let Ok(toot) = Toot::try_from_toot_in(toot) else {
                warn!("failed to parse toot");
                continue;
            };

            if !toots.contains_key(&toot.id) {
                info!("new toot: {}", toot.content);
                toots.insert(toot.id.clone(), toot);
            }
        }
    }
}
