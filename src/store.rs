use uuid::Uuid;
use worker::{kv::KvStore, Error, Result, RouteContext};

use crate::Target;

/// util functions for interfacing with the cloudflare kv store

const TRACKED_URLS_STORE: &str = "QRSTATS_TRACKED_URLS_BY_ID";
const TRACKED_URL_COUNTS: &str = "QRSTATS_TRACKED_URL_COUNTS";

pub(crate) async fn gen_unused_id(kv: &KvStore) -> Result<String> {
    let uuid = Uuid::new_v4().to_string();
    let new_id = uuid.split('-').next().unwrap();
    match kv.get(new_id).text().await? {
        Some(_) => Err(Error::Internal("ID Already Exists, could not retry".into())),
        None => Ok(new_id.to_string()),
    }
}

pub(crate) async fn create_new_target(ctx: &RouteContext<()>, target: &Target) -> Result<String> {
    let kv = ctx.kv(TRACKED_URLS_STORE)?;
    let new_id = gen_unused_id(&kv).await?;
    kv.put(&new_id, target.clone())?.execute().await?;
    Ok(new_id)
}

pub(crate) async fn get_redirect_count(ctx: &RouteContext<()>, id: &str) -> Result<u32> {
    let kv = ctx.kv(TRACKED_URL_COUNTS)?;
    let count = kv
        .get(id)
        .text()
        .await?
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    Ok(count)
}

pub(crate) async fn get_target_by_id(ctx: &RouteContext<()>, id: &str) -> Result<Target> {
    let kv = ctx.kv(TRACKED_URLS_STORE)?;
    match kv.get(id).json::<Target>().await? {
        Some(target) => Ok(target),
        None => Err(Error::Internal("ID Not Found in our System".into())),
    }
}

pub(crate) async fn increment_redirect_count(ctx: &RouteContext<()>, id: &str) -> Result<()> {
    let kv = ctx.kv(TRACKED_URL_COUNTS)?;
    let count = kv
        .get(id)
        .text()
        .await?
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    let new_count = count + 1;
    kv.put(id, new_count.to_string())?.execute().await?;

    Ok(())
}
