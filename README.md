Starting template for serverless http service in WASM using Cloudflare Workers.

## Differences from Cloudflare default template:

- Fully async
- Request/response bodies can be text, json(serialized), or binary
- Non-blocking structured logging
- Deferred tasks that run after response is returned to client

## Getting started

To get started, use

    wrangler generate -t rust PROJECT \
	    https://github.com/stevelr/rustwasm-service-template

where PROJECT is your project name.

## Cloudflare setup

- You'll need a Cloudflare account that is enabled for Workers. This is
  easiest to set up if you also get a domain on Cloudflare; they will
  set up the DNS for you and automatically acquire HTTPS certs.
- Pick a host name for your service. If your domain is "example.com" and
  your service will be at "api.example.com", add 
  `route = "api.example.com/*"` to `wrangler.toml`, 
  and add `api` as an AAAA entry
  in the DNS settings page on the Cloudflare account.
- Important: Set SSL/TLS encryption mode to Full in your Cloudflare
  domain settings.
- Update `wrangler.toml` to set `account_id`, `zone_id`, and `route`
- If you want to require https clients to use TLS 1.3 (more secure), 
  edit `worker/worker.js` and follow the instructions near line 20

## Logging

- The basic logger uses the equivalent of javascript's console.log. When
  using `wrangler dev` or `wrangler preview`, those logs are easy to
  see on the console. For "production" services, check on Cloudflare's log panel.

- if you have a Coralogix account, update `config.toml`
  to set `logger="coralogix"`, and set `api_key`.

- For structured logging with Coralogix, you can get started with a
  free-tier account. Logs can be viewed in real time, either in a browser,
  on the Coralogix dashboard, or using the livetail cli tool.
  `livetail --api-key XXXXX --region eu --format pretty`

