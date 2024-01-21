use worker::{Response, Result};

use crate::{util::serve_html, Stats, Target};

pub(crate) fn stats_view(stats: Stats) -> Result<Response> {
    Response::ok(stats.to_string())
}

pub(crate) fn create_success(target: Target, id: String) -> Result<Response> {
    let redirect_url = format!("/redirect/{}", id);
    let stats_url = "/stats";

    serve_html(
        format!(
            "<h1>Success!</h1>
            <p>New ID for URL(\"{}\") is: {}</p>
            <p><a href=\"{}\">Redirect URL</a> (copy this into your QR generator</p>
            <p><a href=\"{}\">Stats URL</a> (visit here, and enter your ID and password to view your stats)</p>",
            target.url, id, redirect_url, stats_url
        )
        .as_str(),
    )
}
