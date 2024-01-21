use std::fmt::Display;

use serde::{Deserialize, Serialize};
use worker::{event, Context, Env, Request, Response, Result, Router};

mod routes;
pub(crate) mod store;
pub(crate) mod util;
pub(crate) mod views;

use routes::{
    get_create, get_stats_login, index, post_create, post_stats_login, redirect, stats, style,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Target {
    url: String,
    pw_hash: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Stats {
    id: String,
    count: u32,
}

impl Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_json::to_string(self).unwrap().fmt(f)
    }
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/", index)
        .get("/style.css", style)
        .get("/create", get_create)
        .post_async("/create", post_create)
        .get_async("/redirect/:id", redirect)
        .get("/stats", get_stats_login)
        .post_async("/stats", post_stats_login)
        .get_async("/stats/:id", stats)
        .run(req, env)
        .await
}
