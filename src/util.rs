use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use worker::{Error, FormData, FormEntry, Request, Response, Result, Url};

use crate::Target;

pub(crate) fn validate_target(target: &Target) -> Result<()> {
    Url::parse(&target.url)?;
    Ok(())
}

pub(crate) fn set_html_content_type(res: &mut Result<Response>) {
    res.as_mut()
        .map(|res| res.headers_mut().set("Content-Type", "text/html").unwrap())
        .unwrap();
}

pub(crate) fn serve_html(html: &str) -> Result<Response> {
    let mut res = Response::ok(html);
    set_html_content_type(&mut res);
    res
}

pub(crate) fn gen_hash(s: String) -> String {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish().to_string()
}

pub(crate) fn ensure_authed(pw_hash: Option<&String>, target: &Target) -> Result<bool> {
    if let Some(expected) = target.pw_hash.clone() {
        match pw_hash {
            Some(pass) => Ok(expected.eq(&gen_hash(pass.to_string()))),
            None => Ok(false),
        }
    } else {
        // no password required
        return Ok(true);
    }
}

pub(crate) async fn get_target(req: Request) -> Result<Target> {
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
        Some(FormEntry::Field(s)) => Some(gen_hash(s)),
        _ => None,
    };

    Ok(Target {
        url: new_url.clone(),
        pw_hash: pw,
    })
}
