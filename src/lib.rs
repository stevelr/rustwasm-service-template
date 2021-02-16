/// wasm-service template
///
use async_trait::async_trait;
use serde::Serialize;
use service_logging::{log, CoralogixConfig, CoralogixLogger};
use wasm_bindgen::{prelude::*, JsValue};
use wasm_service::{Context, Handler, HandlerReturn, Request, RunContext, Runnable, ServiceConfig};

// compile-time config settings, defined in config.toml
mod config;
use config::CONFIG;

cfg_if::cfg_if! {
    if #[cfg(feature="wee_alloc")] {
        // Use `wee_alloc` as the global allocator.
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[derive(Serialize)]
struct Thing {
    x: u32,
    name: String,
}

struct MyHandler {}
#[async_trait(?Send)]
impl Handler for MyHandler {
    /// Process incoming Request
    async fn handle(&self, req: &Request, ctx: &mut Context) -> Result<(), HandlerReturn> {
        use service_logging::Severity::{Info, Verbose};
        use wasm_service::Method::GET;
        // log all incoming requests
        log!(ctx, Verbose, _:"handler", method: req.method(), url: req.url());
        // Content-Type for responses, unless overridden below
        ctx.response()
            .content_type("text/plain; charset=utf-8")
            .unwrap();

        match (req.method(), req.url().path()) {
            (GET, "/") => {
                log!(ctx, Info, _:"root handler");
                ctx.response().text("OK");
            }
            (GET, "/hello") => {
                log!(ctx, Info, _:"hello");
                ctx.response().text("Hello world!");
            }
            (GET, "/defer") => {
                ctx.defer(Box::new(Task::One(111)));
                ctx.defer(Box::new(Task::Two(222)));
                ctx.response().text("Hello world!");
            }
            (GET, "/json") => {
                // Return a json object
                let an_object = Thing {
                    x: 100,
                    name: "abc".to_string(),
                };
                if let Err(e) = ctx
                    .response()
                    .content_type("application/json")
                    .unwrap()
                    .json(&an_object)
                {
                    // if error occurs during json serialization, report to client
                    ctx.raise_internal_error(Box::new(e))
                }
            }
            (GET, "/favicon.ico") => {
                ctx.response()
                    .content_type("image/x-icon")
                    .unwrap()
                    .header("Cache-control", "max-age:604800,public,immutable")
                    .unwrap()
                    .body(FAVICON.to_vec());
            }
            _ => {
                ctx.response().status(404).text("Not Found");
            }
        }
        Ok(())
    }
}

/// Deferred task types.
/// An enum is used for the example, but anytihng with owned (or static) data
/// that implements Runnable will do.
enum Task {
    One(u64),
    Two(u64),
}

/// Process deferred tasks - this function is called once per task
#[async_trait]
impl Runnable for Task {
    async fn run(&self, ctx: &RunContext) {
        use service_logging::Severity::Info;
        match self {
            Task::One(n) => {
                log!(ctx, Info, text: format!("One: {}", n));
            }
            Task::Two(n) => {
                log!(ctx, Info, text: format!("Two: {}", n));
            }
        }
    }
}

/// Main entry to service worker, called from javascript
#[wasm_bindgen]
pub async fn main_entry(req: JsValue) -> Result<JsValue, JsValue> {
    let logger = match CONFIG.logging.logger.as_ref() {
        //"console" => ConsoleLogger::init(),
        "coralogix" => CoralogixLogger::init(CoralogixConfig {
            api_key: &CONFIG.logging.coralogix.api_key,
            application_name: &CONFIG.logging.coralogix.application_name,
            endpoint: &CONFIG.logging.coralogix.endpoint,
        })
        .map_err(|e| JsValue::from_str(&e.to_string()))?,
        _ => {
            return Err(JsValue::from_str(&format!(
                "Invalid logger configured:\"{}\"",
                CONFIG.logging.logger
            )));
        }
    };
    wasm_service::service_request(
        req,
        ServiceConfig {
            logger,
            handlers: vec![Box::new(MyHandler {})],
            ..Default::default()
        },
    )
    .await
}

// contents of favicon.ico, a rusty-colored R
// To generate this table from your own .ico file, try hexdump -v -e '1/1 "%d,"' < favicon.ico
const FAVICON: [u8; 318] = [
    0, 0, 1, 0, 1, 0, 16, 16, 16, 0, 1, 0, 4, 0, 40, 1, 0, 0, 22, 0, 0, 0, 40, 0, 0, 0, 16, 0, 0,
    0, 32, 0, 0, 0, 1, 0, 4, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0,
    0, 0, 0, 55, 127, 212, 0, 255, 255, 255, 0, 27, 109, 207, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 17, 17, 17, 0, 1, 0, 0, 17, 17, 17, 17, 2, 1, 16, 0, 17,
    17, 17, 16, 34, 1, 18, 34, 17, 17, 17, 0, 2, 17, 18, 34, 17, 17, 16, 2, 33, 17, 18, 34, 17, 17,
    0, 34, 17, 17, 16, 34, 34, 34, 34, 33, 17, 17, 18, 34, 0, 0, 0, 2, 17, 17, 18, 34, 0, 0, 0, 0,
    33, 17, 18, 34, 17, 17, 17, 0, 2, 17, 18, 34, 17, 17, 17, 16, 2, 17, 18, 34, 17, 17, 17, 16, 2,
    17, 18, 34, 17, 17, 17, 0, 34, 17, 16, 0, 2, 34, 34, 34, 34, 17, 0, 34, 34, 2, 2, 2, 33, 17, 0,
    0, 2, 34, 34, 32, 1, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0,
];
