use crate::{prelude::*, toots::Toots};
use warp::{Filter, Rejection, Reply};

async fn serve_toots(toots: Toots) -> Result<()> {
    let toot = warp::path!("toot" / String).map(move |_| {
        let Ok(mut toots) = toots.lock() else {
            error!("failed to lock toots");
            return warp::reply::with_status(
                "failed to lock toots".to_owned(),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            );
        };

        if toots.is_empty() {
            return warp::reply::with_status(
                "no toots".to_owned(),
                warp::http::StatusCode::NOT_FOUND,
            );
        } else {
            let Some((id, toot)) = toots.drain().next() else {
                error!("drain toots");
                return warp::reply::with_status(
                    "failed to drain toots".to_owned(),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                );
            };

            return warp::reply::with_status(toot.content(), warp::http::StatusCode::OK);
        }
    });

    warp::serve(toot).run(([0, 0, 0, 0], 8080)).await;

    Ok(())
}
