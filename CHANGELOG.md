
## v0.2.5

- upgraded to wasm-service 0.4, reqwest 0.11

- Following principles of "Security by Design", the worker requires
  clients to support TLSv1.3. If you need to accept TLSv1.2, adjust
  the check near the top of worker/worker.js

## v0.2.0

- upgraded wasm-service to 0.3, service-logging to 0.4
  - these versions have a slight change in function signatures
    of `Handler.handle` and `Runnable.run`
- added generic page for internal error
- added a favicon.ico
- set default response content type to "text/plain; charset=UTF-8"
