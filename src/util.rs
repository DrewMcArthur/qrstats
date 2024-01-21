use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use worker::{Error, FormData, FormEntry, Request, Response, Result, Url};

use crate::Target;

pub(crate) fn validate_target(target: &mut Target) -> Result<()> {
    if &target.url[..4] != "http" {
        target.url = format!("http://{}", target.url); // default to http if not https;
    }
    Url::parse(&target.url).expect("couldn't parse url");
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

pub(crate) fn ensure_authed(pw: Option<&String>, target: &Target) -> bool {
    if let Some(expected) = target.pw_hash.clone() {
        expected.eq(&gen_hash(pw.unwrap_or(&"".to_string()).to_string()))
    } else {
        // no password required
        true
    }
}

pub(crate) async fn get_target(req: Request) -> Result<Target> {
    let mut req = req;
    let data: FormData = req.form_data().await.expect("couldn't get form data");

    let missing_url_error = Error::Internal("No URL in /create request body".into());
    let bad_url_format_error =
        Error::Internal("URL should be String, not File in /create request body".into());

    let new_url = match data
        .get("url")
        .ok_or(missing_url_error)
        .expect("couldn't get url from formdata")
    {
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
