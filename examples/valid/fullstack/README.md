# Full-stack web demo (WASM frontend + Lullaby HTTP backend, shared module)

A single Lullaby domain module compiled into **two very different targets**: a
WebAssembly frontend that runs in the browser, and an HTTP/1.1 backend that runs
on the TCP socket builtins. Both sides import the same `shared.lby`, so the
priority labels and scores they produce for a given task title are identical.

## Files

- `shared.lby` — the domain module used by BOTH sides (`pub` functions):
  - `classify(title) -> string` — a task title's priority label (`empty` /
    `quick` / `detailed`) derived purely from its length.
  - `priority_score(title) -> i64` — the numeric score for the same title.
  - `is_valid_title(title) -> bool` / `detail_threshold() -> i64` — validation
    and the shared length threshold.

  It is deliberately kept inside the WASM-eligible surface (scalars and strings
  only — string literals, `==`, `len`, arithmetic, `if`), so the *same* source
  compiles to `.wasm` for the browser and runs unchanged in the interpreter-based
  HTTP backend. No `+` concat, `substring`, `to_string`, `list`, `map`, or
  `match` appears in it.

- `frontend.lby` — the browser side. It `import shared`, runs `classify` /
  `priority_score` on three sample titles, and renders the results with the
  JS/DOM host builtins `console_log` and `dom_set_text`. `lullaby wasm` accepts
  every function (shared module included).

- `index.html` — a self-contained host page (no CDN, no remote assets). It loads
  `frontend.wasm` relatively, supplies the `env.console_log` and
  `env.dom_set_text` imports (backed by `console.log` and
  `document.getElementById(id).textContent`), and calls the exported `main()`.

- `backend.lby` — the server side. It `import http` (the reusable framework
  module, copied from the `http_server` example) and `import shared`, and answers
  a `/classify` request with the shared `classify` label and `priority_score`
  for a sample title — the exact values the frontend renders for that title. It
  is a bounded accept loop (like `http_server/server.lby`) so it always
  terminates.

- `http.lby` — the HTTP/1.1 framework module (request parsing, routing, response
  building), the same one the `http_server` example ships.

## Build and open the frontend

Compile the frontend to WebAssembly (writes `frontend.wasm` next to the source):

```
lullaby wasm frontend.lby
```

Then open `index.html`. Because browsers may block `fetch()` of a local
`.wasm` over the `file:` scheme, serve the folder with any static file server
and open the page from `127.0.0.1`. The three task rows fill in with the shared
labels (`quick`, `detailed`, `empty`), the browser console logs the same labels,
and `main()` returns the summed shared priority score (`1 + 3 + 0 = 4`).

You can confirm the exact same values without a browser by running the frontend
on any interpreter backend:

```
lullaby run frontend.lby
```

It prints the console lines, the `id=text` DOM lines, and `4`.

## Run and curl the backend

The backend takes two program arguments: the port and the number of requests to
serve before exiting (both default to `8080` and `1`):

```
lullaby run backend.lby 8080 3
```

That serves three requests on port `8080`, then exits. In another terminal:

```
curl -v 127.0.0.1:8080/
curl -v 127.0.0.1:8080/classify
curl -v 127.0.0.1:8080/nope
```

`/` returns the greeting, `/nope` a `404 Not Found`, and `/classify` returns the
shared classification body for the sample title `Write the design document`:

```
title=Write the design document
label=detailed
score=3
valid=true
```

Because `label` and `score` come from `shared.lby`, they match the frontend's
output for the same title exactly. Run it on any backend:

```
lullaby run --backend ir backend.lby 8080 1
lullaby run --backend bytecode backend.lby 8080 1
```

## The shared module is the point

`classify` and `priority_score` are written once, in `shared.lby`, and executed
on two independent code paths: lowered to WebAssembly and run in the browser, and
run through the Lullaby interpreter inside the HTTP server. The demo proves that
a Lullaby program can share real domain logic across the front-end/back-end
boundary with no duplication.

The Rust test `fullstack_shared_logic_round_trip` in
`crates/lullaby_cli/tests/cli.rs` drives the backend as a real HTTP client on all
three backends and asserts the shared `/classify` body, and
`fullstack_frontend_wasm_matches_shared_logic` emits `frontend.wasm`, asserts the
`\0asm` magic and the compiled entry, and — when `node` is present — instantiates
it with capturing `env.console_log` / `env.dom_set_text` imports and asserts the
rendered labels and the `main` score.
