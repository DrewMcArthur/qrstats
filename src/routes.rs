use crate::{
    store::{create_new_target, get_redirect_count, get_target_by_id, increment_redirect_count},
    util::{ensure_authed, get_target, serve_html, set_html_content_type, validate_target},
    views::{self, stats_view},
    Stats,
};
use worker::{FormData, FormEntry, Request, Response, Result, RouteContext, Url};

pub(crate) fn index(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    serve_html(include_str!("public/index.html"))
}

pub(crate) async fn create(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let target = get_target(req).await?;
    if let Err(e) = validate_target(&target) {
        return Response::error(e.to_string(), 400);
    }

    let new_id = create_new_target(&ctx, &target).await?;

    views::create_success(target, new_id)
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

pub(crate) fn get_stats_login(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let mut res = Response::ok(include_str!("public/stats.html"));
    set_html_content_type(&mut res);
    res
}

pub(crate) async fn post_stats_login(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut req = req;
    let data: FormData = req.form_data().await?;

    let id = match data.get("id") {
        Some(FormEntry::Field(id)) => id,
        _ => return Response::error("Bad Request: ID Not Found", 400),
    };
    let pw_hash = match data.get("pw") {
        Some(FormEntry::Field(pw_hash)) => pw_hash,
        _ => return Response::error("Bad Request: Password Not Found", 400),
    };

    let target = get_target_by_id(&ctx, &id).await?;
    if !ensure_authed(Some(&pw_hash), &target)? {
        return Response::error("Unauthorized", 401);
    }
    let count = get_redirect_count(&ctx, &id).await?;
    stats_view(Stats { id, count })
}

pub(crate) async fn stats(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(id) = ctx.param("id") {
        let target = get_target_by_id(&ctx, id).await?;
        if !ensure_authed(ctx.param("pw"), &target)? {
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
