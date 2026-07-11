// Cross-language data_records suite (JavaScript). Plain-object records + record
// operations. Records are minimal object literals matching the Python/Rust
// structs: Point {x, y}, Rect {w, h}, Money {cents}, Interval {lo, hi},
// Vec3 {x, y, z}.

function manhattan(a, b) {
  return Math.abs(a.x - b.x) + Math.abs(a.y - b.y);
}

function chebyshev(a, b) {
  return Math.max(Math.abs(a.x - b.x), Math.abs(a.y - b.y));
}

function rect_area(r) {
  return r.w * r.h;
}

function rect_perimeter(r) {
  return 2 * (r.w + r.h);
}

function rect_contains(r, px, py) {
  return px >= 0 && py >= 0 && px <= r.w && py <= r.h ? 1 : 0;
}

function money_add(a, b) {
  return { cents: a.cents + b.cents };
}

function money_dollars(m) {
  return Math.trunc(m.cents / 100);
}

function money_cents_part(m) {
  return m.cents % 100;
}

function interval_overlaps(a, b) {
  return a.lo < b.hi && b.lo < a.hi ? 1 : 0;
}

function interval_length(a) {
  return Math.max(a.hi - a.lo, 0);
}

function interval_intersect_len(a, b) {
  const lo = Math.max(a.lo, b.lo);
  const hi = Math.min(a.hi, b.hi);
  return Math.max(hi - lo, 0);
}

function vec3_dot(a, b) {
  return a.x * b.x + a.y * b.y + a.z * b.z;
}

function vec3_add(a, b) {
  return { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z };
}

function vec3_manhattan_norm(v) {
  return Math.abs(v.x) + Math.abs(v.y) + Math.abs(v.z);
}

function midpoint_x(a, b) {
  return Math.trunc((a.x + b.x) / 2);
}

function main() {
  const p = { x: 1, y: 2 };
  const q = { x: 4, y: 6 };
  console.log("manhattan=" + manhattan(p, q));
  console.log("chebyshev=" + chebyshev(p, q));
  const r = { w: 3, h: 5 };
  console.log("rect_area=" + rect_area(r));
  console.log("rect_perimeter=" + rect_perimeter(r));
  console.log("rect_contains=" + rect_contains(r, 2, 4));
  const m = money_add({ cents: 150 }, { cents: 275 });
  console.log("money_dollars=" + money_dollars(m));
  console.log("money_cents_part=" + money_cents_part(m));
  const a = { lo: 1, hi: 5 };
  const b = { lo: 3, hi: 8 };
  console.log("interval_overlaps=" + interval_overlaps(a, b));
  console.log("interval_length=" + interval_length(a));
  console.log("interval_intersect_len=" + interval_intersect_len(a, b));
  const u = { x: 1, y: 2, z: 3 };
  const w = { x: 4, y: 5, z: 6 };
  console.log("vec3_dot=" + vec3_dot(u, w));
  const s = vec3_add(u, w);
  console.log("vec3_add.x=" + s.x);
  console.log("vec3_manhattan_norm=" + vec3_manhattan_norm({ x: -1, y: 2, z: -3 }));
  console.log("midpoint_x=" + midpoint_x(p, q));
}

main();
