/// wasm-service template
///
use async_trait::async_trait;
use serde_json::json;
use service_logging::{log, prelude::*, CoralogixConfig, CoralogixLogger, LogQueue, Severity};
#[cfg(target = "wasm32")]
use service_loging::ConsoleLogger;

use std::{fmt, rc::Rc, sync::Mutex};
use wasm_bindgen::{prelude::*, JsValue};
use wasm_service::{Context, Handler, Method::GET, Runnable};

// compile-time config settings, defined in config.toml
mod config;
use config::CONFIG;

// Errors for this crate
#[derive(Debug)]
enum Error {
    Service(wasm_service::Error),
}

impl From<wasm_service::Error> for Error {
    fn from(e: wasm_service::Error) -> Self {
        Error::Service(e)
    }
}
impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

struct MyHandler {}

#[async_trait(?Send)]
impl Handler<Error> for MyHandler {
    /// Process incoming Request
    async fn handle(&self, ctx: &mut Context) -> Result<(), Error> {
        // log all incoming http hits
        log!(ctx, Severity::Verbose, _:"handler", method: ctx.method(), url: ctx.url());
        // Content-Type for responses, unless overridden below
        ctx.default_content_type("text/plain");

        match (ctx.method(), ctx.url().path()) {
            (GET, "/") => {
                log!(ctx, Severity::Info, _:"root");
                ctx.response().text("OK");
            }
            (GET, "/hello") => {
                log!(ctx, Severity::Info, _:"hello");
                ctx.response().text("Hello world!");
            }
            (GET, "/defer") => {
                log!(ctx, Severity::Info, _:"defer", task_count: 2);
                ctx.defer(Box::new(Task::One(111)));
                ctx.defer(Box::new(Task::Two(222)));
                ctx.response().text("Hello world!");
            }
            (GET, "/json") => {
                log!(ctx, Severity::Info, code:"json");
                // Return a json object
                let an_object = json!({ "x": 1, "found": true, "inner": { "names": ["Alice", "Bob", "Carol"] }});
                ctx.response()
                    .content_type("application/json")?
                    .json(&an_object)?;
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
#[async_trait(?Send)]
impl Runnable for Task {
    async fn run(&self, lq: Rc<Mutex<LogQueue>>) {
        match self {
            Task::One(n) => {
                log!(lq, Severity::Info, text: format!("One: {}", n));
            }
            Task::Two(n) => {
                log!(lq, Severity::Info, text: format!("Two: {}", n));
            }
        }
    }
}

/// Main entry to service worker, called from javascript
#[wasm_bindgen]
pub async fn main_entry(req: JsValue) -> Result<JsValue, JsValue> {
    let logger = match CONFIG.logging.logger.as_ref() {
        #[cfg(target = "wasm32")]
        "console" => ConsoleLogger::init(),
        "coralogix" => CoralogixLogger::init(CoralogixConfig {
            api_key: &CONFIG.logging.coralogix.api_key,
            application_name: &CONFIG.logging.coralogix.application_name,
            endpoint: &CONFIG.logging.coralogix.endpoint,
        })
        .map_err(|e| JsValue::from_str(&e.to_string()))?,
        _ => {
            return Err(JsValue::from_str(&format!(
                "Invalid logger configured:'{}'",
                CONFIG.logging.logger
            )));
        }
    };
    wasm_service::service_request(req, logger, Box::new(MyHandler {})).await
}
