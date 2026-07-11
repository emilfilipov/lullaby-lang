// Cross-language data_records suite (Rust). Structs + record operations.

#[derive(Clone, Copy)]
struct Point { x: i64, y: i64 }
#[derive(Clone, Copy)]
struct Rect { w: i64, h: i64 }
#[derive(Clone, Copy)]
struct Money { cents: i64 }
#[derive(Clone, Copy)]
struct Interval { lo: i64, hi: i64 }
#[derive(Clone, Copy)]
struct Vec3 { x: i64, y: i64, z: i64 }

fn manhattan(a: Point, b: Point) -> i64 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

fn chebyshev(a: Point, b: Point) -> i64 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

fn rect_area(r: Rect) -> i64 {
    r.w * r.h
}

fn rect_perimeter(r: Rect) -> i64 {
    2 * (r.w + r.h)
}

fn rect_contains(r: Rect, px: i64, py: i64) -> i64 {
    if px >= 0 && py >= 0 && px <= r.w && py <= r.h { 1 } else { 0 }
}

fn money_add(a: Money, b: Money) -> Money {
    Money { cents: a.cents + b.cents }
}

fn money_dollars(m: Money) -> i64 {
    m.cents / 100
}

fn money_cents_part(m: Money) -> i64 {
    m.cents % 100
}

fn interval_overlaps(a: Interval, b: Interval) -> i64 {
    if a.lo < b.hi && b.lo < a.hi { 1 } else { 0 }
}

fn interval_length(a: Interval) -> i64 {
    (a.hi - a.lo).max(0)
}

fn interval_intersect_len(a: Interval, b: Interval) -> i64 {
    let lo = a.lo.max(b.lo);
    let hi = a.hi.min(b.hi);
    (hi - lo).max(0)
}

fn vec3_dot(a: Vec3, b: Vec3) -> i64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn vec3_add(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z }
}

fn vec3_manhattan_norm(v: Vec3) -> i64 {
    v.x.abs() + v.y.abs() + v.z.abs()
}

fn midpoint_x(a: Point, b: Point) -> i64 {
    (a.x + b.x) / 2
}

fn main() {
    let p = Point { x: 1, y: 2 };
    let q = Point { x: 4, y: 6 };
    println!("manhattan={}", manhattan(p, q));
    println!("chebyshev={}", chebyshev(p, q));
    let r = Rect { w: 3, h: 5 };
    println!("rect_area={}", rect_area(r));
    println!("rect_perimeter={}", rect_perimeter(r));
    println!("rect_contains={}", rect_contains(r, 2, 4));
    let m = money_add(Money { cents: 150 }, Money { cents: 275 });
    println!("money_dollars={}", money_dollars(m));
    println!("money_cents_part={}", money_cents_part(m));
    let a = Interval { lo: 1, hi: 5 };
    let b = Interval { lo: 3, hi: 8 };
    println!("interval_overlaps={}", interval_overlaps(a, b));
    println!("interval_length={}", interval_length(a));
    println!("interval_intersect_len={}", interval_intersect_len(a, b));
    let u = Vec3 { x: 1, y: 2, z: 3 };
    let w = Vec3 { x: 4, y: 5, z: 6 };
    println!("vec3_dot={}", vec3_dot(u, w));
    let s = vec3_add(u, w);
    println!("vec3_add.x={}", s.x);
    println!("vec3_manhattan_norm={}", vec3_manhattan_norm(Vec3 { x: -1, y: 2, z: -3 }));
    println!("midpoint_x={}", midpoint_x(p, q));
}
