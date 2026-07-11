/* Cross-language services suite (C). The pure decision logic at the core of a
   REST service: status handling, retries/backoff, rate limiting, pagination,
   caching, and queueing — all as plain int64_t functions, no socket layer. */
#include <stdio.h>
#include <stdint.h>

static int64_t min2(int64_t a, int64_t b) {
    return a < b ? a : b;
}

/* HTTP status family: 2..5 for 2xx..5xx, 0 for anything outside 200..599. */
int64_t http_status_class(int64_t code) {
    int64_t cls = code / 100;
    return (cls >= 2 && cls <= 5) ? cls : 0;
}

/* A response is an error once it reaches the 4xx range. */
int64_t status_is_error(int64_t code) {
    return code >= 400 ? 1 : 0;
}

/* Method codes: 0=GET, 1=HEAD, 2=POST, 3=PUT, 4=DELETE. GET/HEAD are safe. */
int64_t method_is_safe_code(int64_t m) {
    return (m == 0 || m == 1) ? 1 : 0;
}

/* Exponential backoff base * 2^attempt, capped at 60000 ms. */
int64_t retry_backoff_ms(int64_t attempt, int64_t base) {
    int64_t d = base;
    for (int64_t i = 0; i < attempt; i++) {
        d *= 2;
        if (d >= 60000) return 60000;
    }
    return d > 60000 ? 60000 : d;
}

/* Token bucket: a request is allowed when the bucket holds at least its cost. */
int64_t rate_limit_allow(int64_t tokens, int64_t cost) {
    return tokens >= cost ? 1 : 0;
}

/* Bucket level after spending `cost` and refilling, clamped to `cap`. */
int64_t tokens_after(int64_t tokens, int64_t cost, int64_t refill, int64_t cap) {
    return min2(cap, tokens - cost + refill);
}

/* Number of pages needed to hold `total` items at `per` per page (ceil). */
int64_t pagination_total_pages(int64_t total, int64_t per) {
    if (per <= 0) return 0;
    return (total + per - 1) / per;
}

/* Zero-based item offset of a 1-based page. */
int64_t pagination_offset(int64_t page, int64_t per) {
    return (page - 1) * per;
}

/* Whether a 1-based page is followed by at least one more item. */
int64_t pagination_has_next(int64_t page, int64_t per, int64_t total) {
    return (page * per < total) ? 1 : 0;
}

/* New LRU recency position after a hit (hits + 1), bounded to the table size. */
int64_t lru_new_position(int64_t hits) {
    return min2(hits + 1, 1024);
}

/* A cache entry is fresh while it is younger than its TTL. */
int64_t cache_ttl_valid(int64_t now, int64_t created, int64_t ttl) {
    return (now - created < ttl) ? 1 : 0;
}

/* Scheduling priority 0..3 from a status code (5xx highest, 2xx lowest). */
int64_t priority_from_code(int64_t code) {
    int64_t cls = http_status_class(code);
    if (cls == 5) return 0;
    if (cls == 4) return 1;
    if (cls == 3) return 2;
    return 3;
}

/* Ring-buffer advance: (head + 1) mod cap. */
int64_t queue_next_index(int64_t head, int64_t cap) {
    return (head + 1) % cap;
}

/* A session stays valid until it is idle longer than its timeout. */
int64_t session_valid(int64_t now, int64_t last, int64_t timeout) {
    return (now - last < timeout) ? 1 : 0;
}

/* Throttle spacing: spread `requests` evenly across `window_ms`, given a fixed
   ceiling of 100 requests per window. */
int64_t throttle_delay_ms(int64_t requests, int64_t window_ms) {
    return requests * window_ms / 100;
}

/* Content-type family from an extension code: 0=html, 1=json, 2=text-like
   (text/css/js), 3=binary. */
int64_t content_type_class(int64_t ext_code) {
    if (ext_code == 0) return 0;
    if (ext_code == 1) return 1;
    if (ext_code == 2) return 2;
    if (ext_code == 3) return 2;
    if (ext_code == 4) return 2;
    return 3;
}

/* Stop retrying once the attempt count reaches the maximum. */
int64_t retry_should_give_up(int64_t attempt, int64_t max) {
    return attempt >= max ? 1 : 0;
}

/* Deterministic jittered backoff: exponential backoff plus a seed-derived offset. */
int64_t jitter_backoff(int64_t attempt, int64_t base, int64_t seed) {
    return retry_backoff_ms(attempt, base) + seed % base;
}

int main(void) {
    printf("http_status_class(404)=%lld\n", (long long)http_status_class(404));
    printf("status_is_error(503)=%lld\n", (long long)status_is_error(503));
    printf("method_is_safe_code(0)=%lld\n", (long long)method_is_safe_code(0));
    printf("retry_backoff_ms(5,100)=%lld\n", (long long)retry_backoff_ms(5, 100));
    printf("rate_limit_allow(3,5)=%lld\n", (long long)rate_limit_allow(3, 5));
    printf("tokens_after(3,5,4,10)=%lld\n", (long long)tokens_after(3, 5, 4, 10));
    printf("pagination_total_pages(95,10)=%lld\n", (long long)pagination_total_pages(95, 10));
    printf("pagination_offset(4,10)=%lld\n", (long long)pagination_offset(4, 10));
    printf("pagination_has_next(4,10,95)=%lld\n", (long long)pagination_has_next(4, 10, 95));
    printf("lru_new_position(7)=%lld\n", (long long)lru_new_position(7));
    printf("cache_ttl_valid(100,90,30)=%lld\n", (long long)cache_ttl_valid(100, 90, 30));
    printf("priority_from_code(500)=%lld\n", (long long)priority_from_code(500));
    printf("queue_next_index(7,8)=%lld\n", (long long)queue_next_index(7, 8));
    printf("session_valid(100,40,30)=%lld\n", (long long)session_valid(100, 40, 30));
    printf("throttle_delay_ms(20,1000)=%lld\n", (long long)throttle_delay_ms(20, 1000));
    printf("content_type_class(3)=%lld\n", (long long)content_type_class(3));
    printf("retry_should_give_up(5,5)=%lld\n", (long long)retry_should_give_up(5, 5));
    printf("jitter_backoff(3,100,250)=%lld\n", (long long)jitter_backoff(3, 100, 250));
    return 0;
}
