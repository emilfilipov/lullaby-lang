// Cross-language services suite (Rust). The pure decision logic at the core of a
// REST service: status handling, retries/backoff, rate limiting, pagination,
// caching, and queueing — all as plain i64 functions, no socket layer.
#![allow(dead_code)]

// HTTP status family: 2..5 for 2xx..5xx, 0 for anything outside 200..599.
fn http_status_class(code: i64) -> i64 {
    let cls = code / 100;
    if cls >= 2 && cls <= 5 { cls } else { 0 }
}

// A response is an error once it reaches the 4xx range.
fn status_is_error(code: i64) -> i64 {
    if code >= 400 { 1 } else { 0 }
}

// Method codes: 0=GET, 1=HEAD, 2=POST, 3=PUT, 4=DELETE. GET/HEAD are safe.
fn method_is_safe_code(m: i64) -> i64 {
    if m == 0 || m == 1 { 1 } else { 0 }
}

// Exponential backoff base * 2^attempt, capped at 60000 ms.
fn retry_backoff_ms(attempt: i64, base: i64) -> i64 {
    let mut d = base;
    let mut i = 0;
    while i < attempt {
        d *= 2;
        if d >= 60000 {
            return 60000;
        }
        i += 1;
    }
    if d > 60000 { 60000 } else { d }
}

// Token bucket: a request is allowed when the bucket holds at least its cost.
fn rate_limit_allow(tokens: i64, cost: i64) -> i64 {
    if tokens >= cost { 1 } else { 0 }
}

// Bucket level after spending `cost` and refilling, clamped to `cap`.
fn tokens_after(tokens: i64, cost: i64, refill: i64, cap: i64) -> i64 {
    (tokens - cost + refill).min(cap)
}

// Number of pages needed to hold `total` items at `per` per page (ceil).
fn pagination_total_pages(total: i64, per: i64) -> i64 {
    if per <= 0 {
        return 0;
    }
    (total + per - 1) / per
}

// Zero-based item offset of a 1-based page.
fn pagination_offset(page: i64, per: i64) -> i64 {
    (page - 1) * per
}

// Whether a 1-based page is followed by at least one more item.
fn pagination_has_next(page: i64, per: i64, total: i64) -> i64 {
    if page * per < total { 1 } else { 0 }
}

// New LRU recency position after a hit (hits + 1), bounded to the table size.
fn lru_new_position(hits: i64) -> i64 {
    (hits + 1).min(1024)
}

// A cache entry is fresh while it is younger than its TTL.
fn cache_ttl_valid(now: i64, created: i64, ttl: i64) -> i64 {
    if now - created < ttl { 1 } else { 0 }
}

// Scheduling priority 0..3 from a status code (5xx highest, 2xx lowest).
fn priority_from_code(code: i64) -> i64 {
    match http_status_class(code) {
        5 => 0,
        4 => 1,
        3 => 2,
        _ => 3,
    }
}

// Ring-buffer advance: (head + 1) mod cap.
fn queue_next_index(head: i64, cap: i64) -> i64 {
    (head + 1) % cap
}

// A session stays valid until it is idle longer than its timeout.
fn session_valid(now: i64, last: i64, timeout: i64) -> i64 {
    if now - last < timeout { 1 } else { 0 }
}

// Throttle spacing: spread `requests` evenly across `window_ms`, given a fixed
// ceiling of 100 requests per window.
fn throttle_delay_ms(requests: i64, window_ms: i64) -> i64 {
    requests * window_ms / 100
}

// Content-type family from an extension code: 0=html, 1=json, 2=text-like
// (text/css/js), 3=binary.
fn content_type_class(ext_code: i64) -> i64 {
    match ext_code {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 2,
        4 => 2,
        _ => 3,
    }
}

// Stop retrying once the attempt count reaches the maximum.
fn retry_should_give_up(attempt: i64, max: i64) -> i64 {
    if attempt >= max { 1 } else { 0 }
}

// Deterministic jittered backoff: exponential backoff plus a seed-derived offset.
fn jitter_backoff(attempt: i64, base: i64, seed: i64) -> i64 {
    retry_backoff_ms(attempt, base) + seed % base
}

fn main() {
    println!("http_status_class(404)={}", http_status_class(404));
    println!("status_is_error(503)={}", status_is_error(503));
    println!("method_is_safe_code(0)={}", method_is_safe_code(0));
    println!("retry_backoff_ms(5,100)={}", retry_backoff_ms(5, 100));
    println!("rate_limit_allow(3,5)={}", rate_limit_allow(3, 5));
    println!("tokens_after(3,5,4,10)={}", tokens_after(3, 5, 4, 10));
    println!("pagination_total_pages(95,10)={}", pagination_total_pages(95, 10));
    println!("pagination_offset(4,10)={}", pagination_offset(4, 10));
    println!("pagination_has_next(4,10,95)={}", pagination_has_next(4, 10, 95));
    println!("lru_new_position(7)={}", lru_new_position(7));
    println!("cache_ttl_valid(100,90,30)={}", cache_ttl_valid(100, 90, 30));
    println!("priority_from_code(500)={}", priority_from_code(500));
    println!("queue_next_index(7,8)={}", queue_next_index(7, 8));
    println!("session_valid(100,40,30)={}", session_valid(100, 40, 30));
    println!("throttle_delay_ms(20,1000)={}", throttle_delay_ms(20, 1000));
    println!("content_type_class(3)={}", content_type_class(3));
    println!("retry_should_give_up(5,5)={}", retry_should_give_up(5, 5));
    println!("jitter_backoff(3,100,250)={}", jitter_backoff(3, 100, 250));
}
