use worker::{Response, Result};

use crate::{util::serve_html, Stats, Target};

pub(crate) fn stats_view(stats: Stats) -> Result<Response> {
    let body = format!(
        "<h1>Stats</h1>
        <a href=\"/\">Home</a>
        <a href=\"/stats\">Back to Login</a>
        <h2>ID</h2>
        <p>{}</p>
        <h2>Count</h2>
        <p>{}</p>",
        stats.id, stats.count
    );
    let html = format!(include_str!("./public/layout.html"), body);
    serve_html(html.as_str())
}

pub(crate) fn create_success(target: Target, id: String) -> Result<Response> {
    let redirect_url = format!("/redirect/{}", id);
    let stats_url = format!("/stats/{}", id);

    let body = format!(
            "<h1>Success!</h1>
            <a href=\"/\">Home</a>
            <p>New ID for URL(\"{}\") is: {}</p>
            <p><a href=\"{}\">Redirect URL</a> (copy this into your QR generator</p>
            <p><a href=\"{}\">Stats URL</a> (visit here, and enter your ID and password to view your stats)</p>",
            target.url, id, redirect_url, stats_url
        );
    let html = format!(include_str!("./public/layout.html"), body);
    serve_html(html.as_str())
}
