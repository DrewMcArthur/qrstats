use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use worker::{kv::KvStore, *};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Target {
    url: String,
    pw_hash: Option<String>,
}

const TRACKED_URLS_STORE: &str = "QRSTATS_TRACKED_URLS_BY_ID";
const TRACKED_URL_COUNTS: &str = "QRSTATS_TRACKED_URL_COUNTS";

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/", index)
        .post_async("/create", create)
        .get_async("/redirect/:id", redirect)
        .get("/stats", stats_login)
        .get_async("/stats/:id", stats)
        .run(req, env)
        .await
}

fn index(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    serve_html(include_str!("public/index.html"))
}

async fn create(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let target = get_target(req).await?;

    let kv = ctx.kv(TRACKED_URLS_STORE)?;
    let new_id = gen_unused_id(&kv).await?;
    kv.put(&new_id, target.clone())?.execute().await?;

    let redirect_url = format!("/redirect/{}", new_id);
    let stats_url = format!("/stats/{}", new_id);

    Response::ok(format!(
        "Success! New ID for URL(\"{}\") is: {}\nRedirect URL: {}\nStats URL: {}",
        target.url, new_id, redirect_url, stats_url
    ))
}

async fn get_target(req: Request) -> Result<Target> {
    let mut req = req;
    let data: FormData = req.form_data().await?;

    let missing_url_error = Error::Internal("No URL in /create request body".into());
    let bad_url_format_error =
        Error::Internal("URL should be String, not File in /create request body".into());

    let new_url = match data.get("url").ok_or(missing_url_error)? {
        FormEntry::Field(s) => s,
        FormEntry::File(_) => return Err(bad_url_format_error),
    };

    let pw = match data.get("password") {
        Some(FormEntry::Field(s)) => Some(gen_hash(s)),
        _ => None,
    };

    Ok(Target {
        url: new_url.clone(),
        pw_hash: pw,
    })
}

async fn gen_unused_id(kv: &KvStore) -> Result<String> {
    let uuid = Uuid::new_v4().to_string();
    let new_id = uuid.split("-").next().unwrap();
    match kv.get(&new_id).text().await? {
        Some(_) => return Err(Error::Internal("ID Already Exists, could not retry".into())),
        None => return Ok(new_id.to_string()),
    }
}

async fn redirect(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(id) = ctx.param("id") {
        let success = increment_redirect_count(&ctx, id);
        let url = get_target_by_id(&ctx, id).await?.url;
        success.await?;
        Response::redirect(Url::parse(&url)?)
    } else {
        Response::error("Bad Request: No ID in URL", 400)
    }
}

async fn increment_redirect_count(ctx: &RouteContext<()>, id: &String) -> Result<()> {
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

async fn get_target_by_id(ctx: &RouteContext<()>, id: &String) -> Result<Target> {
    let kv = ctx.kv(TRACKED_URLS_STORE)?;
    match kv.get(id).text().await? {
        Some(target) => Ok(serde_json::from_str(&target)?),
        None => Err(Error::Internal("ID Not Found in our System".into())),
    }
}

fn stats_login(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let mut res = Response::ok(include_str!("public/stats.html"));
    set_html_content_type(&mut res);
    res
}

async fn stats(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(id) = ctx.param("id") {
        let target = get_target_by_id(&ctx, id).await?;
        if !ensure_authed(&ctx, &target)? {
            return Response::error("Unauthorized", 401);
        }
        let count = get_redirect_count(&ctx, id).await?;
        Response::ok(format!(
            "Hello, World! redirect counts for id {} = {}",
            id, count
        ))
    } else {
        Response::error("Bad Request: ID Not Found URL", 400)
    }
}

fn ensure_authed(ctx: &RouteContext<()>, target: &Target) -> Result<bool> {
    if let Some(pw) = target.pw_hash.clone() {
        match ctx.param("password") {
            Some(pass) => Ok(pw.eq(&gen_hash(pass.to_string()))),
            None => Ok(false),
        }
    } else {
        // no password required
        return Ok(true);
    }
}

fn set_html_content_type(res: &mut Result<Response>) {
    res.as_mut()
        .map(|res| res.headers_mut().set("Content-Type", "text/html").unwrap())
        .unwrap();
}

fn serve_html(html: &str) -> Result<Response> {
    let mut res = Response::ok(html);
    set_html_content_type(&mut res);
    res
}

fn gen_hash(s: String) -> String {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish().to_string()
}

async fn get_redirect_count(ctx: &RouteContext<()>, id: &String) -> Result<u32> {
    let kv = ctx.kv(TRACKED_URL_COUNTS)?;
    let count = kv
        .get(id)
        .text()
        .await?
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    Ok(count)
}
