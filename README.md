# qrstats

this is a simple app built to enable simple statistics on url traffic (mainly for QR codes). It was build in rust on cloudflare's serverless workers platform.

it was cloned from [the cloudflare/workers-sdk/templates/experimental/worker-rust repository](https://github.com/cloudflare/workers-sdk/tree/main/templates/experimental/worker-rust), and made by following [this tutorial.](https://developers.cloudflare.com/workers/runtime-apis/webassembly/rust/)

## routes

the app only has a handful of routes that it serves, so here's an outline.  routes are defined in [src/lib.rs](https://github.com/DrewMcArthur/qrstats/blob/2e82945bb15d31f05d3499650dbb0394a84cd049/src/lib.rs#L36)

### GET /index

the index page is a simple landing page, which explains how the app works and allows the user to create a redirect for their URL, as well as an optional password to secure their stats, as well as some other configuration. this is managed via a simple HTML form, which POSTs the request to a qrstats-worker, and is handled there.

### GET /create

a form for creating a new tracked URL.  requires a URL, and optionally accepts a custom ID and password.

### POST /create

sends the data from the form on the above page to the server to be added to the KV store.  returns either `views::create_success` or `views::create_error`

### GET /redirect/:id

fetches the URL for the given ID, increments the saved count for that tracked URL, and issues an HTTP Redirect response to the stored URL.

### GET /stats

this page allows the user to "login", either via their id along with an optional password.

### POST /stats

this returns the stats for the ID from the form above.

### GET /stats/:id

this attempts to authenticate, and either returns a similar response to POST /stats, or redirects to the GET /stats login page.
