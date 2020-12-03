// worker.js
// send fetch events to rust wasm
addEventListener('fetch', event => {
  event.respondWith(handleRequest(event))
})

// Forward incoming requests to Rust Handler.
//
// Deferred tasks, if any, are passed in a Promise to event.waitUntil,
// so they can be processed after the response is returned to the client.
async function handleRequest(event) {
    const { main_entry, run_deferred } = wasm_bindgen;
    await wasm_bindgen(wasm);
    const request = event.request;
    var result, response;
    try {
        if (request.cf !== undefined) {
            const tlsVersion = request.cf.tlsVersion
            // Allow only TLS 1.2 and 1.3
            if (tlsVersion != "TLSv1.2" && tlsVersion != "TLSv1.3") {
                return new Response(
                    "Please use TLS version 1.2 or higher.", { status: 403, }
                );
            }
        }

        // fully read body (synchronously) before calling Rust handler
        let input = new Map();
        input.set("body", new Uint8Array(await request.arrayBuffer()));
        input.set("method", request.method);
        input.set("url", request.url);
        input.set("headers", request.headers);
        input.set("event", event);

        // call rust handler, put results into a Response object
        result = await main_entry(input);
        var body_bin = result.get("body"); // Uint8Array
        response = new Response(body_bin, {
            status: result.get("status"),
            headers: result.get("headers"),
        });
    } catch(error) {
        response = new Response("Error:" + error, {status: 200});
    }
    return response;
}
