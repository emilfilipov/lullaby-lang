/* Cross-language geometry suite (C). Integer geometry on Point structs. */
#include <stdio.h>
#include <stdint.h>

typedef struct { int64_t x, y; } Point;

static int64_t abs_val(int64_t n) {
    return n < 0 ? -n : n;
}

int64_t dist_sq(Point a, Point b) {
    int64_t dx = a.x - b.x;
    int64_t dy = a.y - b.y;
    return dx * dx + dy * dy;
}

int64_t manhattan(Point a, Point b) {
    return abs_val(a.x - b.x) + abs_val(a.y - b.y);
}

int64_t cross(int64_t ax, int64_t ay, int64_t bx, int64_t by) {
    return ax * by - ay * bx;
}

int64_t dot(int64_t ax, int64_t ay, int64_t bx, int64_t by) {
    return ax * bx + ay * by;
}

int64_t triangle_area2(Point a, Point b, Point c) {
    return cross(b.x - a.x, b.y - a.y, c.x - a.x, c.y - a.y);
}

int64_t is_ccw(Point a, Point b, Point c) {
    return triangle_area2(a, b, c) > 0 ? 1 : 0;
}

int64_t collinear(Point a, Point b, Point c) {
    return triangle_area2(a, b, c) == 0 ? 1 : 0;
}

int64_t on_segment_x(Point a, Point b, int64_t px) {
    int64_t lo = a.x < b.x ? a.x : b.x;
    int64_t hi = a.x > b.x ? a.x : b.x;
    return (px >= lo && px <= hi) ? 1 : 0;
}

int64_t rect_area(int64_t w, int64_t h) {
    return w * h;
}

int64_t perimeter_rect(int64_t w, int64_t h) {
    return 2 * (w + h);
}

int64_t point_in_rect(int64_t px, int64_t py, int64_t w, int64_t h) {
    return (px >= 0 && py >= 0 && px <= w && py <= h) ? 1 : 0;
}

int64_t midpoint_x(Point a, Point b) {
    return (a.x + b.x) / 2;
}

int64_t midpoint_y(Point a, Point b) {
    return (a.y + b.y) / 2;
}

int64_t taxicab_circle_points(int64_t r) {
    return r > 0 ? 4 * r : 1;
}

int64_t quadrant(int64_t px, int64_t py) {
    if (px == 0 || py == 0) return 0;
    if (px > 0 && py > 0) return 1;
    if (px < 0 && py > 0) return 2;
    if (px < 0 && py < 0) return 3;
    return 4;
}

int main(void) {
    Point a = { 0, 0 }, b = { 3, 4 }, c = { 6, 8 };
    printf("dist_sq=%lld\n", (long long)dist_sq(a, b));
    printf("manhattan=%lld\n", (long long)manhattan(a, b));
    printf("cross=%lld\n", (long long)cross(3, 4, 6, 8));
    printf("dot=%lld\n", (long long)dot(3, 4, 6, 8));
    Point d = { 4, 0 };
    printf("triangle_area2=%lld\n", (long long)triangle_area2(a, b, d));
    printf("is_ccw=%lld\n", (long long)is_ccw(a, b, d));
    printf("collinear=%lld\n", (long long)collinear(a, b, c));
    printf("on_segment_x=%lld\n", (long long)on_segment_x(a, b, 2));
    printf("rect_area=%lld\n", (long long)rect_area(3, 5));
    printf("perimeter_rect=%lld\n", (long long)perimeter_rect(3, 5));
    printf("point_in_rect=%lld\n", (long long)point_in_rect(2, 4, 3, 5));
    printf("midpoint_x=%lld\n", (long long)midpoint_x(a, c));
    printf("midpoint_y=%lld\n", (long long)midpoint_y(a, c));
    printf("taxicab_circle_points=%lld\n", (long long)taxicab_circle_points(3));
    printf("quadrant=%lld\n", (long long)quadrant(-2, 5));
    return 0;
}
