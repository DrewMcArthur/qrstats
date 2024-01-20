# qrstats

this is a simple app built to enable simple statistics on url traffic (mainly for QR codes). It was build in rust on cloudflare's serverless workers platform.

it was cloned from [the cloudflare/workers-sdk/templates/experimental/worker-rust repository](https://github.com/cloudflare/workers-sdk/tree/main/templates/experimental/worker-rust), and made by following [this tutorial.](https://developers.cloudflare.com/workers/runtime-apis/webassembly/rust/)

## routes

the app only has a handful of routes that it serves, so here's an outline.

### GET /index

the index page is a simple landing page, which explains how the app works and allows the user to create a redirect for their URL, as well as an optional password to secure their stats, as well as some other configuration. this is managed via a simple HTML form, which POSTs the request to a qrstats-worker, and is handled there.

### GET /stats

this page allows the user to "login", either via their url, or the ID they were given, along with an optional password.
