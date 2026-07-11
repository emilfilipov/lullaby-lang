// Cross-language data_records suite (C++). Structs + record operations.
#include <cstdint>
#include <iostream>

struct Point { std::int64_t x, y; };
struct Rect { std::int64_t w, h; };
struct Money { std::int64_t cents; };
struct Interval { std::int64_t lo, hi; };
struct Vec3 { std::int64_t x, y, z; };

std::int64_t abs_val(std::int64_t n) {
    return n < 0 ? -n : n;
}

std::int64_t manhattan(Point a, Point b) {
    return abs_val(a.x - b.x) + abs_val(a.y - b.y);
}

std::int64_t chebyshev(Point a, Point b) {
    std::int64_t dx = abs_val(a.x - b.x);
    std::int64_t dy = abs_val(a.y - b.y);
    return dx > dy ? dx : dy;
}

std::int64_t rect_area(Rect r) {
    return r.w * r.h;
}

std::int64_t rect_perimeter(Rect r) {
    return 2 * (r.w + r.h);
}

std::int64_t rect_contains(Rect r, std::int64_t px, std::int64_t py) {
    return (px >= 0 && py >= 0 && px <= r.w && py <= r.h) ? 1 : 0;
}

Money money_add(Money a, Money b) {
    return Money{ a.cents + b.cents };
}

std::int64_t money_dollars(Money m) {
    return m.cents / 100;
}

std::int64_t money_cents_part(Money m) {
    return m.cents % 100;
}

std::int64_t interval_overlaps(Interval a, Interval b) {
    return (a.lo < b.hi && b.lo < a.hi) ? 1 : 0;
}

std::int64_t interval_length(Interval a) {
    std::int64_t n = a.hi - a.lo;
    return n < 0 ? 0 : n;
}

std::int64_t interval_intersect_len(Interval a, Interval b) {
    std::int64_t lo = a.lo > b.lo ? a.lo : b.lo;
    std::int64_t hi = a.hi < b.hi ? a.hi : b.hi;
    std::int64_t n = hi - lo;
    return n < 0 ? 0 : n;
}

std::int64_t vec3_dot(Vec3 a, Vec3 b) {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}

Vec3 vec3_add(Vec3 a, Vec3 b) {
    return Vec3{ a.x + b.x, a.y + b.y, a.z + b.z };
}

std::int64_t vec3_manhattan_norm(Vec3 v) {
    return abs_val(v.x) + abs_val(v.y) + abs_val(v.z);
}

std::int64_t midpoint_x(Point a, Point b) {
    return (a.x + b.x) / 2;
}

int main() {
    Point p{ 1, 2 }, q{ 4, 6 };
    std::cout << "manhattan=" << manhattan(p, q) << "\n";
    std::cout << "chebyshev=" << chebyshev(p, q) << "\n";
    Rect r{ 3, 5 };
    std::cout << "rect_area=" << rect_area(r) << "\n";
    std::cout << "rect_perimeter=" << rect_perimeter(r) << "\n";
    std::cout << "rect_contains=" << rect_contains(r, 2, 4) << "\n";
    Money m = money_add(Money{ 150 }, Money{ 275 });
    std::cout << "money_dollars=" << money_dollars(m) << "\n";
    std::cout << "money_cents_part=" << money_cents_part(m) << "\n";
    Interval a{ 1, 5 }, b{ 3, 8 };
    std::cout << "interval_overlaps=" << interval_overlaps(a, b) << "\n";
    std::cout << "interval_length=" << interval_length(a) << "\n";
    std::cout << "interval_intersect_len=" << interval_intersect_len(a, b) << "\n";
    Vec3 u{ 1, 2, 3 }, w{ 4, 5, 6 };
    std::cout << "vec3_dot=" << vec3_dot(u, w) << "\n";
    Vec3 s = vec3_add(u, w);
    std::cout << "vec3_add.x=" << s.x << "\n";
    std::cout << "vec3_manhattan_norm=" << vec3_manhattan_norm(Vec3{ -1, 2, -3 }) << "\n";
    std::cout << "midpoint_x=" << midpoint_x(p, q) << "\n";
    return 0;
}
