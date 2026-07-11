// Cross-language services suite (JavaScript). The pure decision logic at the core
// of a REST service: status handling, retries/backoff, rate limiting, pagination,
// caching, and queueing — all as plain integer functions, no socket layer.

// HTTP status family: 2..5 for 2xx..5xx, 0 for anything outside 200..599.
function http_status_class(code) {
    const cls = Math.trunc(code / 100);
    return cls >= 2 && cls <= 5 ? cls : 0;
}

// A response is an error once it reaches the 4xx range.
function status_is_error(code) {
    return code >= 400 ? 1 : 0;
}

// Method codes: 0=GET, 1=HEAD, 2=POST, 3=PUT, 4=DELETE. GET/HEAD are safe.
function method_is_safe_code(m) {
    return m === 0 || m === 1 ? 1 : 0;
}

// Exponential backoff base * 2^attempt, capped at 60000 ms.
function retry_backoff_ms(attempt, base) {
    let d = base;
    for (let i = 0; i < attempt; i++) {
        d *= 2;
        if (d >= 60000) return 60000;
    }
    return d > 60000 ? 60000 : d;
}

// Token bucket: a request is allowed when the bucket holds at least its cost.
function rate_limit_allow(tokens, cost) {
    return tokens >= cost ? 1 : 0;
}

// Bucket level after spending `cost` and refilling, clamped to `cap`.
function tokens_after(tokens, cost, refill, cap) {
    return Math.min(cap, tokens - cost + refill);
}

// Number of pages needed to hold `total` items at `per` per page (ceil).
function pagination_total_pages(total, per) {
    if (per <= 0) return 0;
    return Math.trunc((total + per - 1) / per);
}

// Zero-based item offset of a 1-based page.
function pagination_offset(page, per) {
    return (page - 1) * per;
}

// Whether a 1-based page is followed by at least one more item.
function pagination_has_next(page, per, total) {
    return page * per < total ? 1 : 0;
}

// New LRU recency position after a hit (hits + 1), bounded to the table size.
function lru_new_position(hits) {
    return Math.min(hits + 1, 1024);
}

// A cache entry is fresh while it is younger than its TTL.
function cache_ttl_valid(now, created, ttl) {
    return now - created < ttl ? 1 : 0;
}

// Scheduling priority 0..3 from a status code (5xx highest, 2xx lowest).
function priority_from_code(code) {
    const cls = http_status_class(code);
    if (cls === 5) return 0;
    if (cls === 4) return 1;
    if (cls === 3) return 2;
    return 3;
}

// Ring-buffer advance: (head + 1) mod cap.
function queue_next_index(head, cap) {
    return (head + 1) % cap;
}

// A session stays valid until it is idle longer than its timeout.
function session_valid(now, last, timeout) {
    return now - last < timeout ? 1 : 0;
}

// Throttle spacing: spread `requests` evenly across `window_ms`, given a fixed
// ceiling of 100 requests per window.
function throttle_delay_ms(requests, window_ms) {
    return Math.trunc((requests * window_ms) / 100);
}

// Content-type family from an extension code: 0=html, 1=json, 2=text-like
// (text/css/js), 3=binary.
function content_type_class(ext_code) {
    if (ext_code === 0) return 0;
    if (ext_code === 1) return 1;
    if (ext_code === 2) return 2;
    if (ext_code === 3) return 2;
    if (ext_code === 4) return 2;
    return 3;
}

// Stop retrying once the attempt count reaches the maximum.
function retry_should_give_up(attempt, max) {
    return attempt >= max ? 1 : 0;
}

// Deterministic jittered backoff: exponential backoff plus a seed-derived offset.
function jitter_backoff(attempt, base, seed) {
    return retry_backoff_ms(attempt, base) + (seed % base);
}

function main() {
    console.log("http_status_class(404)=" + http_status_class(404));
    console.log("status_is_error(503)=" + status_is_error(503));
    console.log("method_is_safe_code(0)=" + method_is_safe_code(0));
    console.log("retry_backoff_ms(5,100)=" + retry_backoff_ms(5, 100));
    console.log("rate_limit_allow(3,5)=" + rate_limit_allow(3, 5));
    console.log("tokens_after(3,5,4,10)=" + tokens_after(3, 5, 4, 10));
    console.log("pagination_total_pages(95,10)=" + pagination_total_pages(95, 10));
    console.log("pagination_offset(4,10)=" + pagination_offset(4, 10));
    console.log("pagination_has_next(4,10,95)=" + pagination_has_next(4, 10, 95));
    console.log("lru_new_position(7)=" + lru_new_position(7));
    console.log("cache_ttl_valid(100,90,30)=" + cache_ttl_valid(100, 90, 30));
    console.log("priority_from_code(500)=" + priority_from_code(500));
    console.log("queue_next_index(7,8)=" + queue_next_index(7, 8));
    console.log("session_valid(100,40,30)=" + session_valid(100, 40, 30));
    console.log("throttle_delay_ms(20,1000)=" + throttle_delay_ms(20, 1000));
    console.log("content_type_class(3)=" + content_type_class(3));
    console.log("retry_should_give_up(5,5)=" + retry_should_give_up(5, 5));
    console.log("jitter_backoff(3,100,250)=" + jitter_backoff(3, 100, 250));
}

main();
