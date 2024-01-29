use crate::{prelude::*, toots::Toots};
use axum::{
    body::Body,
    extract::Extension,
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::{get, Router},
};
use tower::ServiceBuilder;
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};

pub async fn serve_toots(toots: Toots) -> Result<()> {
    let middleware_stack = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .layer(Extension(toots));

    let app: Router = Router::new()
        .route("/", get(toot_mp3))
        .layer(middleware_stack)
        .layer(TraceLayer::new_for_http());

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind to port 8080");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn toot_mp3(Extension(toots): Extension<Toots>) -> impl IntoResponse {
    let toot_text = {
        let mut toots = toots.lock().await;
        if let Some(toot_content) = toots
            .values()
            .filter(|toot| !toot.served)
            .next()
            .map(|toot| toot.content.clone())
        {
            toots
                .values_mut()
                .filter(|toot| toot.content == toot_content)
                .for_each(|toot| {
                    info!("serving toot: {}", toot.content);
                    toot.served = true
                });

            info!(
                "{} of {} toots served",
                toots.iter().filter(|(_, toot)| toot.served).count(),
                toots.len()
            );

            Some(toot_content)
        } else {
            None
        }
    }
    .unwrap_or(std::env::var("NOT_FOUND_TEXT").unwrap_or_else(|_| "No new toots.".into()));

    let client = reqwest::Client::new();
    let voice_id = std::env::var("VOICE_ID").unwrap_or_else(|_| "t0jbNlBVZ17f02VDIeMI".into());
    let body = ElevenlabsBody::new(toot_text);
    let api_key = std::env::var("ELEVEN_LABS_API_KEY").expect("ELEVEN_LABS_API_KEY is not set");

    let resp = client
        .post(&format!(
            "https://api.elevenlabs.io/v1/text-to-speech/{voice_id}"
        ))
        .header("xi-api-key", api_key)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(Error::BadRequest(format!(
            "failed to request tts from vendor: {}",
            resp.status()
        )));
    }

    let bytes = resp.bytes().await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "audio/mp3")
        .body(Body::from(bytes))
        .expect("failed to build response"))
}

#[derive(serde::Serialize, Clone)]
pub struct ElevenlabsBody {
    text: String,
    model_id: String,
}

impl ElevenlabsBody {
    fn new(text: String) -> Self {
        Self {
            text,
            model_id: std::env::var("MODEL_ID").unwrap_or_else(|_| "eleven_multilingual_v2".into()),
        }
    }
}
