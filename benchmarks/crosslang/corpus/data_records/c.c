/* Cross-language data_records suite (C). Structs + record operations. */
#include <stdio.h>
#include <stdint.h>

typedef struct { int64_t x, y; } Point;
typedef struct { int64_t w, h; } Rect;
typedef struct { int64_t cents; } Money;
typedef struct { int64_t lo, hi; } Interval;
typedef struct { int64_t x, y, z; } Vec3;

static int64_t abs_val(int64_t n) {
    return n < 0 ? -n : n;
}

int64_t manhattan(Point a, Point b) {
    return abs_val(a.x - b.x) + abs_val(a.y - b.y);
}

int64_t chebyshev(Point a, Point b) {
    int64_t dx = abs_val(a.x - b.x);
    int64_t dy = abs_val(a.y - b.y);
    return dx > dy ? dx : dy;
}

int64_t rect_area(Rect r) {
    return r.w * r.h;
}

int64_t rect_perimeter(Rect r) {
    return 2 * (r.w + r.h);
}

int64_t rect_contains(Rect r, int64_t px, int64_t py) {
    return (px >= 0 && py >= 0 && px <= r.w && py <= r.h) ? 1 : 0;
}

Money money_add(Money a, Money b) {
    Money m = { a.cents + b.cents };
    return m;
}

int64_t money_dollars(Money m) {
    return m.cents / 100;
}

int64_t money_cents_part(Money m) {
    return m.cents % 100;
}

int64_t interval_overlaps(Interval a, Interval b) {
    return (a.lo < b.hi && b.lo < a.hi) ? 1 : 0;
}

int64_t interval_length(Interval a) {
    int64_t n = a.hi - a.lo;
    return n < 0 ? 0 : n;
}

int64_t interval_intersect_len(Interval a, Interval b) {
    int64_t lo = a.lo > b.lo ? a.lo : b.lo;
    int64_t hi = a.hi < b.hi ? a.hi : b.hi;
    int64_t n = hi - lo;
    return n < 0 ? 0 : n;
}

int64_t vec3_dot(Vec3 a, Vec3 b) {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}

Vec3 vec3_add(Vec3 a, Vec3 b) {
    Vec3 r = { a.x + b.x, a.y + b.y, a.z + b.z };
    return r;
}

int64_t vec3_manhattan_norm(Vec3 v) {
    return abs_val(v.x) + abs_val(v.y) + abs_val(v.z);
}

int64_t midpoint_x(Point a, Point b) {
    return (a.x + b.x) / 2;
}

int main(void) {
    Point p = { 1, 2 }, q = { 4, 6 };
    printf("manhattan=%lld\n", (long long)manhattan(p, q));
    printf("chebyshev=%lld\n", (long long)chebyshev(p, q));
    Rect r = { 3, 5 };
    printf("rect_area=%lld\n", (long long)rect_area(r));
    printf("rect_perimeter=%lld\n", (long long)rect_perimeter(r));
    printf("rect_contains=%lld\n", (long long)rect_contains(r, 2, 4));
    Money m = money_add((Money){ 150 }, (Money){ 275 });
    printf("money_dollars=%lld\n", (long long)money_dollars(m));
    printf("money_cents_part=%lld\n", (long long)money_cents_part(m));
    Interval a = { 1, 5 }, b = { 3, 8 };
    printf("interval_overlaps=%lld\n", (long long)interval_overlaps(a, b));
    printf("interval_length=%lld\n", (long long)interval_length(a));
    printf("interval_intersect_len=%lld\n", (long long)interval_intersect_len(a, b));
    Vec3 u = { 1, 2, 3 }, w = { 4, 5, 6 };
    printf("vec3_dot=%lld\n", (long long)vec3_dot(u, w));
    Vec3 s = vec3_add(u, w);
    printf("vec3_add.x=%lld\n", (long long)s.x);
    printf("vec3_manhattan_norm=%lld\n", (long long)vec3_manhattan_norm((Vec3){ -1, 2, -3 }));
    printf("midpoint_x=%lld\n", (long long)midpoint_x(p, q));
    return 0;
}
