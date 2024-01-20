use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    Router::new()
        .get("/", index)
        .post("/create", create)
        .get("/redirect/{id}", redirect)
        .get("/stats", stats_login)
        .post("/stats/{id}", stats)
        .run(req, env)
        .await
}

fn index(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    Response::ok("Hello, Index!")
}

fn create(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    Response::ok("Hello, Created World!")
}

fn redirect(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    Response::ok("Hello, Redirected World!")
}

fn stats_login(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    Response::ok("Hello, Stats Login World!")
}

fn stats(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    Response::ok("Hello, Stats World!")
}
