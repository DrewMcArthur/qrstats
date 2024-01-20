use serde::{Deserialize, Serialize};
use worker::{event, Context, Env, Request, Response, Result, Router};

mod routes;
pub(crate) mod store;
pub(crate) mod util;

use routes::{create, index, redirect, stats, stats_login};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Target {
    url: String,
    pw_hash: Option<String>,
}

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
