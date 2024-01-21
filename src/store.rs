use uuid::Uuid;
use worker::{console_log, kv::KvStore, Error, Result, RouteContext};

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
    let kv = ctx
        .kv(TRACKED_URLS_STORE)
        .expect("couldn't get URLS kv store");

    let new_id = match &target.id {
        Some(id) => id.to_owned(),
        None => gen_unused_id(&kv).await.expect("could not generate new id"),
    };

    console_log!("creating new target with id={}: {:?}", new_id, target);
    kv.put(&new_id, target.clone())
        .expect("couldn't put new target")
        .execute()
        .await
        .expect("couldn't execute kv put");

    Ok(new_id.clone())
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
    console_log!("fetching target by id: {}", id);
    let kv = ctx
        .kv(TRACKED_URLS_STORE)
        .expect("couldn't get URLS kv store");

    match kv
        .get(id)
        .json::<Target>()
        .await
        .expect("couldn't parse target from kvstore")
    {
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
