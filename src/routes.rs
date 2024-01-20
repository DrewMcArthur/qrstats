use crate::{
    store::{create_new_target, get_redirect_count, get_target_by_id, increment_redirect_count},
    util::{ensure_authed, get_target, serve_html, set_html_content_type, validate_target},
};
use worker::{Request, Response, Result, RouteContext, Url};

pub(crate) fn index(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    serve_html(include_str!("public/index.html"))
}

pub(crate) async fn create(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let target = get_target(req).await?;
    if let Err(e) = validate_target(&target) {
        return Response::error(e.to_string(), 400);
    }

    let new_id = create_new_target(&ctx, &target).await?;

    let redirect_url = format!("/redirect/{}", new_id);
    let stats_url = format!("/stats/{}", new_id);

    Response::ok(format!(
        "Success! New ID for URL(\"{}\") is: {}\nRedirect URL: {}\nStats URL: {}",
        target.url, new_id, redirect_url, stats_url
    ))
}

pub(crate) async fn redirect(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(id) = ctx.param("id") {
        let success = increment_redirect_count(&ctx, id);
        let url = get_target_by_id(&ctx, id).await?.url;
        success.await?;
        Response::redirect(Url::parse(&url)?)
    } else {
        Response::error("Bad Request: No ID in URL", 400)
    }
}

pub(crate) fn stats_login(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let mut res = Response::ok(include_str!("public/stats.html"));
    set_html_content_type(&mut res);
    res
}

pub(crate) async fn stats(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
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
