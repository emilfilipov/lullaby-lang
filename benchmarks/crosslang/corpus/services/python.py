# Cross-language services suite (Python). The pure decision logic at the core of
# a REST service: status handling, retries/backoff, rate limiting, pagination,
# caching, and queueing — all as plain int functions, no socket layer.


# HTTP status family: 2..5 for 2xx..5xx, 0 for anything outside 200..599.
def http_status_class(code):
    cls = code // 100
    return cls if 2 <= cls <= 5 else 0


# A response is an error once it reaches the 4xx range.
def status_is_error(code):
    return 1 if code >= 400 else 0


# Method codes: 0=GET, 1=HEAD, 2=POST, 3=PUT, 4=DELETE. GET/HEAD are safe.
def method_is_safe_code(m):
    return 1 if m in (0, 1) else 0


# Exponential backoff base * 2^attempt, capped at 60000 ms.
def retry_backoff_ms(attempt, base):
    d = base
    for _ in range(attempt):
        d *= 2
        if d >= 60000:
            return 60000
    return 60000 if d > 60000 else d


# Token bucket: a request is allowed when the bucket holds at least its cost.
def rate_limit_allow(tokens, cost):
    return 1 if tokens >= cost else 0


# Bucket level after spending `cost` and refilling, clamped to `cap`.
def tokens_after(tokens, cost, refill, cap):
    return min(cap, tokens - cost + refill)


# Number of pages needed to hold `total` items at `per` per page (ceil).
def pagination_total_pages(total, per):
    if per <= 0:
        return 0
    return (total + per - 1) // per


# Zero-based item offset of a 1-based page.
def pagination_offset(page, per):
    return (page - 1) * per


# Whether a 1-based page is followed by at least one more item.
def pagination_has_next(page, per, total):
    return 1 if page * per < total else 0


# New LRU recency position after a hit (hits + 1), bounded to the table size.
def lru_new_position(hits):
    return min(hits + 1, 1024)


# A cache entry is fresh while it is younger than its TTL.
def cache_ttl_valid(now, created, ttl):
    return 1 if now - created < ttl else 0


# Scheduling priority 0..3 from a status code (5xx highest, 2xx lowest).
def priority_from_code(code):
    cls = http_status_class(code)
    if cls == 5:
        return 0
    if cls == 4:
        return 1
    if cls == 3:
        return 2
    return 3


# Ring-buffer advance: (head + 1) mod cap.
def queue_next_index(head, cap):
    return (head + 1) % cap


# A session stays valid until it is idle longer than its timeout.
def session_valid(now, last, timeout):
    return 1 if now - last < timeout else 0


# Throttle spacing: spread `requests` evenly across `window_ms`, given a fixed
# ceiling of 100 requests per window.
def throttle_delay_ms(requests, window_ms):
    return requests * window_ms // 100


# Content-type family from an extension code: 0=html, 1=json, 2=text-like
# (text/css/js), 3=binary.
def content_type_class(ext_code):
    if ext_code == 0:
        return 0
    if ext_code == 1:
        return 1
    if ext_code == 2:
        return 2
    if ext_code == 3:
        return 2
    if ext_code == 4:
        return 2
    return 3


# Stop retrying once the attempt count reaches the maximum.
def retry_should_give_up(attempt, max):
    return 1 if attempt >= max else 0


# Deterministic jittered backoff: exponential backoff plus a seed-derived offset.
def jitter_backoff(attempt, base, seed):
    return retry_backoff_ms(attempt, base) + seed % base


def main():
    print("http_status_class(404)=" + str(http_status_class(404)))
    print("status_is_error(503)=" + str(status_is_error(503)))
    print("method_is_safe_code(0)=" + str(method_is_safe_code(0)))
    print("retry_backoff_ms(5,100)=" + str(retry_backoff_ms(5, 100)))
    print("rate_limit_allow(3,5)=" + str(rate_limit_allow(3, 5)))
    print("tokens_after(3,5,4,10)=" + str(tokens_after(3, 5, 4, 10)))
    print("pagination_total_pages(95,10)=" + str(pagination_total_pages(95, 10)))
    print("pagination_offset(4,10)=" + str(pagination_offset(4, 10)))
    print("pagination_has_next(4,10,95)=" + str(pagination_has_next(4, 10, 95)))
    print("lru_new_position(7)=" + str(lru_new_position(7)))
    print("cache_ttl_valid(100,90,30)=" + str(cache_ttl_valid(100, 90, 30)))
    print("priority_from_code(500)=" + str(priority_from_code(500)))
    print("queue_next_index(7,8)=" + str(queue_next_index(7, 8)))
    print("session_valid(100,40,30)=" + str(session_valid(100, 40, 30)))
    print("throttle_delay_ms(20,1000)=" + str(throttle_delay_ms(20, 1000)))
    print("content_type_class(3)=" + str(content_type_class(3)))
    print("retry_should_give_up(5,5)=" + str(retry_should_give_up(5, 5)))
    print("jitter_backoff(3,100,250)=" + str(jitter_backoff(3, 100, 250)))


if __name__ == "__main__":
    main()
