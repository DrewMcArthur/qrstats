use crate::{
    store::{create_new_target, get_redirect_count, get_target_by_id, increment_redirect_count},
    util::{ensure_authed, get_target, serve_html, validate_target},
    views::{self, stats_view},
    Stats,
};
use worker::{FormData, FormEntry, Request, Response, Result, RouteContext, Url};

pub(crate) fn style(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let mut res = Response::ok(include_str!("public/style.css"))?;
    res.headers_mut().set("Content-Type", "text/css")?;
    Ok(res)
}

pub(crate) fn index(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    serve_html(include_str!("public/index.html"))
}

pub(crate) fn get_create(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    serve_html(include_str!("public/create.html"))
}

pub(crate) async fn post_create(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut target = get_target(req).await.expect("couldn't get target");

    if let Err(e) = validate_target(&mut target) {
        return Response::error(e.to_string(), 400);
    }

    let new_id = create_new_target(&ctx, &target)
        .await
        .expect("couldn't create target");

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
    serve_html(include_str!("public/stats.html"))
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
    if !ensure_authed(Some(&pw_hash), &target) {
        return Response::error("Unauthorized", 401);
    }
    let count = get_redirect_count(&ctx, &id).await?;
    stats_view(Stats { id, count })
}

pub(crate) async fn stats(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let req_url = req.url().expect("couldn't get request url");
    let origin = req_url.origin().ascii_serialization();

    if let Some(id) = ctx.param("id") {
        match get_target_by_id(&ctx, id).await {
            Ok(target) => {
                if ensure_authed(ctx.param("pw"), &target) {
                    let count = get_redirect_count(&ctx, id).await.unwrap_or(0);
                    views::stats_view(Stats {
                        id: id.to_string(),
                        count,
                    })
                } else {
                    Response::redirect(
                        Url::parse_with_params(format!("{}/stats", origin).as_str(), &[("id", id)])
                            .expect("couldn't create URL"),
                    )
                }
            }
            Err(e) => Response::error(
                format!("error fetching data from store: {}", e.to_string()),
                500,
            ),
        }
    } else {
        Response::redirect(Url::parse(format!("{}/stats", origin).as_str())?)
    }
}
