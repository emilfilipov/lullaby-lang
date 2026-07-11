// Cross-language geometry suite (JavaScript). Integer geometry on Point structs.

function makePoint(x, y) {
  return { x: x, y: y };
}

function dist_sq(a, b) {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  return dx * dx + dy * dy;
}

function manhattan(a, b) {
  return Math.abs(a.x - b.x) + Math.abs(a.y - b.y);
}

function cross(ax, ay, bx, by) {
  return ax * by - ay * bx;
}

function dot(ax, ay, bx, by) {
  return ax * bx + ay * by;
}

function triangle_area2(a, b, c) {
  return cross(b.x - a.x, b.y - a.y, c.x - a.x, c.y - a.y);
}

function is_ccw(a, b, c) {
  return triangle_area2(a, b, c) > 0 ? 1 : 0;
}

function collinear(a, b, c) {
  return triangle_area2(a, b, c) === 0 ? 1 : 0;
}

function on_segment_x(a, b, px) {
  const lo = Math.min(a.x, b.x);
  const hi = Math.max(a.x, b.x);
  return px >= lo && px <= hi ? 1 : 0;
}

function rect_area(w, h) {
  return w * h;
}

function perimeter_rect(w, h) {
  return 2 * (w + h);
}

function point_in_rect(px, py, w, h) {
  return px >= 0 && py >= 0 && px <= w && py <= h ? 1 : 0;
}

function midpoint_x(a, b) {
  return Math.trunc((a.x + b.x) / 2);
}

function midpoint_y(a, b) {
  return Math.trunc((a.y + b.y) / 2);
}

function taxicab_circle_points(r) {
  return r > 0 ? 4 * r : 1;
}

function quadrant(px, py) {
  if (px === 0 || py === 0) {
    return 0;
  }
  if (px > 0 && py > 0) {
    return 1;
  }
  if (px < 0 && py > 0) {
    return 2;
  }
  if (px < 0 && py < 0) {
    return 3;
  }
  return 4;
}

function main() {
  const a = makePoint(0, 0);
  const b = makePoint(3, 4);
  const c = makePoint(6, 8);
  console.log("dist_sq=" + dist_sq(a, b));
  console.log("manhattan=" + manhattan(a, b));
  console.log("cross=" + cross(3, 4, 6, 8));
  console.log("dot=" + dot(3, 4, 6, 8));
  const d = makePoint(4, 0);
  console.log("triangle_area2=" + triangle_area2(a, b, d));
  console.log("is_ccw=" + is_ccw(a, b, d));
  console.log("collinear=" + collinear(a, b, c));
  console.log("on_segment_x=" + on_segment_x(a, b, 2));
  console.log("rect_area=" + rect_area(3, 5));
  console.log("perimeter_rect=" + perimeter_rect(3, 5));
  console.log("point_in_rect=" + point_in_rect(2, 4, 3, 5));
  console.log("midpoint_x=" + midpoint_x(a, c));
  console.log("midpoint_y=" + midpoint_y(a, c));
  console.log("taxicab_circle_points=" + taxicab_circle_points(3));
  console.log("quadrant=" + quadrant(-2, 5));
}

main();
