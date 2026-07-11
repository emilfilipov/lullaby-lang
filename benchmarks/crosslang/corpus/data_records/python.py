"""Cross-language data_records suite (Python). Dataclasses + record operations."""
from dataclasses import dataclass


@dataclass
class Point:
    x: int
    y: int


@dataclass
class Rect:
    w: int
    h: int


@dataclass
class Money:
    cents: int


@dataclass
class Interval:
    lo: int
    hi: int


@dataclass
class Vec3:
    x: int
    y: int
    z: int


def manhattan(a: Point, b: Point) -> int:
    return abs(a.x - b.x) + abs(a.y - b.y)


def chebyshev(a: Point, b: Point) -> int:
    return max(abs(a.x - b.x), abs(a.y - b.y))


def rect_area(r: Rect) -> int:
    return r.w * r.h


def rect_perimeter(r: Rect) -> int:
    return 2 * (r.w + r.h)


def rect_contains(r: Rect, px: int, py: int) -> int:
    return 1 if 0 <= px <= r.w and 0 <= py <= r.h else 0


def money_add(a: Money, b: Money) -> Money:
    return Money(a.cents + b.cents)


def money_dollars(m: Money) -> int:
    return m.cents // 100


def money_cents_part(m: Money) -> int:
    return m.cents % 100


def interval_overlaps(a: Interval, b: Interval) -> int:
    return 1 if a.lo < b.hi and b.lo < a.hi else 0


def interval_length(a: Interval) -> int:
    return max(a.hi - a.lo, 0)


def interval_intersect_len(a: Interval, b: Interval) -> int:
    lo = max(a.lo, b.lo)
    hi = min(a.hi, b.hi)
    return max(hi - lo, 0)


def vec3_dot(a: Vec3, b: Vec3) -> int:
    return a.x * b.x + a.y * b.y + a.z * b.z


def vec3_add(a: Vec3, b: Vec3) -> Vec3:
    return Vec3(a.x + b.x, a.y + b.y, a.z + b.z)


def vec3_manhattan_norm(v: Vec3) -> int:
    return abs(v.x) + abs(v.y) + abs(v.z)


def midpoint_x(a: Point, b: Point) -> int:
    return (a.x + b.x) // 2


def main() -> None:
    p, q = Point(1, 2), Point(4, 6)
    print("manhattan=" + str(manhattan(p, q)))
    print("chebyshev=" + str(chebyshev(p, q)))
    r = Rect(3, 5)
    print("rect_area=" + str(rect_area(r)))
    print("rect_perimeter=" + str(rect_perimeter(r)))
    print("rect_contains=" + str(rect_contains(r, 2, 4)))
    m = money_add(Money(150), Money(275))
    print("money_dollars=" + str(money_dollars(m)))
    print("money_cents_part=" + str(money_cents_part(m)))
    a, b = Interval(1, 5), Interval(3, 8)
    print("interval_overlaps=" + str(interval_overlaps(a, b)))
    print("interval_length=" + str(interval_length(a)))
    print("interval_intersect_len=" + str(interval_intersect_len(a, b)))
    u, w = Vec3(1, 2, 3), Vec3(4, 5, 6)
    print("vec3_dot=" + str(vec3_dot(u, w)))
    s = vec3_add(u, w)
    print("vec3_add.x=" + str(s.x))
    print("vec3_manhattan_norm=" + str(vec3_manhattan_norm(Vec3(-1, 2, -3))))
    print("midpoint_x=" + str(midpoint_x(p, q)))


if __name__ == "__main__":
    main()
