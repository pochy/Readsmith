use crate::{
    db::{feed_repository, now_ts},
    services::feed_fetcher,
    state::AppState,
};
use futures::{StreamExt, stream};
use std::time::Duration;

pub async fn run(state: AppState) {
    loop {
        if let Err(err) = tick(&state).await {
            tracing::warn!(error = %err, "feed pull worker tick failed");
        }
        tokio::time::sleep(state.inner.config.pull_interval).await;
    }
}

async fn tick(state: &AppState) -> anyhow::Result<()> {
    let limit = state.inner.config.pull_concurrency.max(1) as i64 * 4;
    let feeds = feed_repository::due_for_refresh(&state.inner.db, now_ts(), limit).await?;
    let concurrency = state.inner.config.pull_concurrency.max(1);
    stream::iter(feeds)
        .for_each_concurrent(concurrency, |feed| {
            let state = state.clone();
            async move {
                match feed_fetcher::refresh_feed(&state, &feed).await {
                    Ok(count) => tracing::info!(
                        feed_id = feed.id,
                        inserted_or_updated = count,
                        "feed refreshed"
                    ),
                    Err(err) => {
                        tracing::warn!(feed_id = feed.id, error = %err, "feed refresh failed")
                    }
                }
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        })
        .await;
    Ok(())
}
