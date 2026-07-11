// Cross-language physics suite (JavaScript). Integer physics & unit conversions.
// All math is scaled-integer (no floats): temperatures are tenths (x10),
// distances/speeds use fixed scale factors, energy uses g = 10 m/s^2.

function distance_uniform(v, t) {
  return v * t;
}

function distance_accel(v, t, a) {
  return v * t + Math.trunc(a * t * t / 2);
}

function final_velocity(v, a, t) {
  return v + a * t;
}

function kinetic_energy_x2(m, v) {
  return m * v * v;
}

function momentum(m, v) {
  return m * v;
}

function force(m, a) {
  return m * a;
}

function work(f, d) {
  return f * d;
}

function power_watts(work, t) {
  return Math.trunc(work / t);
}

function pressure(f, area) {
  if (area === 0) {
    return 0;
  }
  return Math.trunc(f / area);
}

function density(mass, vol) {
  return Math.trunc(mass / vol);
}

function celsius_to_fahrenheit_x10(c) {
  return c * 18 + 320;
}

function fahrenheit_to_celsius_x10(f) {
  return Math.trunc((f - 32) * 100 / 18);
}

function celsius_to_kelvin_x10(c) {
  return c * 10 + 2732;
}

function km_to_miles_scaled(km) {
  return Math.trunc(km * 621 / 1000);
}

function miles_to_km_scaled(mi) {
  return Math.trunc(mi * 1609 / 1000);
}

function mps_to_kmh(mps) {
  return Math.trunc(mps * 36 / 10);
}

function gravity_fall_dist(t) {
  return 5 * t * t;
}

function potential_energy(m, h) {
  return m * 10 * h;
}

function main() {
  console.log("distance_uniform=" + distance_uniform(60, 2));
  console.log("distance_accel=" + distance_accel(10, 4, 3));
  console.log("final_velocity=" + final_velocity(5, 3, 4));
  console.log("kinetic_energy_x2=" + kinetic_energy_x2(4, 5));
  console.log("momentum=" + momentum(8, 3));
  console.log("force=" + force(10, 9));
  console.log("work=" + work(50, 4));
  console.log("power_watts=" + power_watts(1000, 4));
  console.log("pressure=" + pressure(200, 4));
  console.log("density=" + density(1000, 8));
  console.log("celsius_to_fahrenheit_x10=" + celsius_to_fahrenheit_x10(25));
  console.log("fahrenheit_to_celsius_x10=" + fahrenheit_to_celsius_x10(77));
  console.log("celsius_to_kelvin_x10=" + celsius_to_kelvin_x10(25));
  console.log("km_to_miles_scaled=" + km_to_miles_scaled(100));
  console.log("miles_to_km_scaled=" + miles_to_km_scaled(100));
  console.log("mps_to_kmh=" + mps_to_kmh(10));
  console.log("gravity_fall_dist=" + gravity_fall_dist(3));
  console.log("potential_energy=" + potential_energy(2, 10));
}

main();
