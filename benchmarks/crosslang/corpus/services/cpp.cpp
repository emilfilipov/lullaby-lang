// Cross-language services suite (C++). The pure decision logic at the core of a
// REST service: status handling, retries/backoff, rate limiting, pagination,
// caching, and queueing — all as plain int64_t functions, no socket layer.
#include <cstdint>
#include <algorithm>
#include <iostream>

// HTTP status family: 2..5 for 2xx..5xx, 0 for anything outside 200..599.
std::int64_t http_status_class(std::int64_t code) {
    std::int64_t cls = code / 100;
    return (cls >= 2 && cls <= 5) ? cls : 0;
}

// A response is an error once it reaches the 4xx range.
std::int64_t status_is_error(std::int64_t code) {
    return code >= 400 ? 1 : 0;
}

// Method codes: 0=GET, 1=HEAD, 2=POST, 3=PUT, 4=DELETE. GET/HEAD are safe.
std::int64_t method_is_safe_code(std::int64_t m) {
    return (m == 0 || m == 1) ? 1 : 0;
}

// Exponential backoff base * 2^attempt, capped at 60000 ms.
std::int64_t retry_backoff_ms(std::int64_t attempt, std::int64_t base) {
    std::int64_t d = base;
    for (std::int64_t i = 0; i < attempt; i++) {
        d *= 2;
        if (d >= 60000) return 60000;
    }
    return d > 60000 ? 60000 : d;
}

// Token bucket: a request is allowed when the bucket holds at least its cost.
std::int64_t rate_limit_allow(std::int64_t tokens, std::int64_t cost) {
    return tokens >= cost ? 1 : 0;
}

// Bucket level after spending `cost` and refilling, clamped to `cap`.
std::int64_t tokens_after(std::int64_t tokens, std::int64_t cost, std::int64_t refill, std::int64_t cap) {
    return std::min(cap, tokens - cost + refill);
}

// Number of pages needed to hold `total` items at `per` per page (ceil).
std::int64_t pagination_total_pages(std::int64_t total, std::int64_t per) {
    if (per <= 0) return 0;
    return (total + per - 1) / per;
}

// Zero-based item offset of a 1-based page.
std::int64_t pagination_offset(std::int64_t page, std::int64_t per) {
    return (page - 1) * per;
}

// Whether a 1-based page is followed by at least one more item.
std::int64_t pagination_has_next(std::int64_t page, std::int64_t per, std::int64_t total) {
    return (page * per < total) ? 1 : 0;
}

// New LRU recency position after a hit (hits + 1), bounded to the table size.
std::int64_t lru_new_position(std::int64_t hits) {
    return std::min<std::int64_t>(hits + 1, 1024);
}

// A cache entry is fresh while it is younger than its TTL.
std::int64_t cache_ttl_valid(std::int64_t now, std::int64_t created, std::int64_t ttl) {
    return (now - created < ttl) ? 1 : 0;
}

// Scheduling priority 0..3 from a status code (5xx highest, 2xx lowest).
std::int64_t priority_from_code(std::int64_t code) {
    std::int64_t cls = http_status_class(code);
    if (cls == 5) return 0;
    if (cls == 4) return 1;
    if (cls == 3) return 2;
    return 3;
}

// Ring-buffer advance: (head + 1) mod cap.
std::int64_t queue_next_index(std::int64_t head, std::int64_t cap) {
    return (head + 1) % cap;
}

// A session stays valid until it is idle longer than its timeout.
std::int64_t session_valid(std::int64_t now, std::int64_t last, std::int64_t timeout) {
    return (now - last < timeout) ? 1 : 0;
}

// Throttle spacing: spread `requests` evenly across `window_ms`, given a fixed
// ceiling of 100 requests per window.
std::int64_t throttle_delay_ms(std::int64_t requests, std::int64_t window_ms) {
    return requests * window_ms / 100;
}

// Content-type family from an extension code: 0=html, 1=json, 2=text-like
// (text/css/js), 3=binary.
std::int64_t content_type_class(std::int64_t ext_code) {
    if (ext_code == 0) return 0;
    if (ext_code == 1) return 1;
    if (ext_code == 2) return 2;
    if (ext_code == 3) return 2;
    if (ext_code == 4) return 2;
    return 3;
}

// Stop retrying once the attempt count reaches the maximum.
std::int64_t retry_should_give_up(std::int64_t attempt, std::int64_t max) {
    return attempt >= max ? 1 : 0;
}

// Deterministic jittered backoff: exponential backoff plus a seed-derived offset.
std::int64_t jitter_backoff(std::int64_t attempt, std::int64_t base, std::int64_t seed) {
    return retry_backoff_ms(attempt, base) + seed % base;
}

int main() {
    std::cout << "http_status_class(404)=" << http_status_class(404) << "\n";
    std::cout << "status_is_error(503)=" << status_is_error(503) << "\n";
    std::cout << "method_is_safe_code(0)=" << method_is_safe_code(0) << "\n";
    std::cout << "retry_backoff_ms(5,100)=" << retry_backoff_ms(5, 100) << "\n";
    std::cout << "rate_limit_allow(3,5)=" << rate_limit_allow(3, 5) << "\n";
    std::cout << "tokens_after(3,5,4,10)=" << tokens_after(3, 5, 4, 10) << "\n";
    std::cout << "pagination_total_pages(95,10)=" << pagination_total_pages(95, 10) << "\n";
    std::cout << "pagination_offset(4,10)=" << pagination_offset(4, 10) << "\n";
    std::cout << "pagination_has_next(4,10,95)=" << pagination_has_next(4, 10, 95) << "\n";
    std::cout << "lru_new_position(7)=" << lru_new_position(7) << "\n";
    std::cout << "cache_ttl_valid(100,90,30)=" << cache_ttl_valid(100, 90, 30) << "\n";
    std::cout << "priority_from_code(500)=" << priority_from_code(500) << "\n";
    std::cout << "queue_next_index(7,8)=" << queue_next_index(7, 8) << "\n";
    std::cout << "session_valid(100,40,30)=" << session_valid(100, 40, 30) << "\n";
    std::cout << "throttle_delay_ms(20,1000)=" << throttle_delay_ms(20, 1000) << "\n";
    std::cout << "content_type_class(3)=" << content_type_class(3) << "\n";
    std::cout << "retry_should_give_up(5,5)=" << retry_should_give_up(5, 5) << "\n";
    std::cout << "jitter_backoff(3,100,250)=" << jitter_backoff(3, 100, 250) << "\n";
    return 0;
}
