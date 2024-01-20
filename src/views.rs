use worker::{Response, Result};

pub(crate) fn stats_view(id: &str, count: u32) -> Result<Response> {
    Response::ok(format!("Stats for {}: {} redirects!", id, count))
}
