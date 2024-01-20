use serde::{Deserialize, Serialize};
use uuid::Uuid;
use worker::{kv::KvStore, *};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Target {
    url: String,
    password: Option<String>,
}

const TRACKED_URLS_STORE: &str = "QRSTATS_TRACKED_URLS_BY_ID";

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/", index)
        .post_async("/create", create)
        .get("/redirect/{id}", redirect)
        .get("/stats", stats_login)
        .post("/stats/{id}", stats)
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

    Response::ok(format!(
        "Success! New ID for URL(\"{}\") is: {}",
        target.url, new_id
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
        Some(FormEntry::Field(s)) => Some(s),
        _ => None,
    };

    Ok(Target {
        url: new_url.clone(),
        password: pw.map(|pw| pw.to_string()),
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

fn redirect(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::ok("Hello, Redirected World!")
}

fn stats_login(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let mut res = Response::ok(include_str!("public/stats.html"));
    set_html_content_type(&mut res);
    res
}

fn stats(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::ok("Hello, Stats World!")
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
