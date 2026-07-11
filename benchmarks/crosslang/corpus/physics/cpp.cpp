// Cross-language physics suite (C++). Integer physics & unit conversions.
// All math is scaled-integer (no floats): temperatures are tenths (x10),
// distances/speeds use fixed scale factors, energy uses g = 10 m/s^2.
#include <cstdint>
#include <iostream>

int64_t distance_uniform(int64_t v, int64_t t) {
    return v * t;
}

int64_t distance_accel(int64_t v, int64_t t, int64_t a) {
    return v * t + a * t * t / 2;
}

int64_t final_velocity(int64_t v, int64_t a, int64_t t) {
    return v + a * t;
}

int64_t kinetic_energy_x2(int64_t m, int64_t v) {
    return m * v * v;
}

int64_t momentum(int64_t m, int64_t v) {
    return m * v;
}

int64_t force(int64_t m, int64_t a) {
    return m * a;
}

int64_t work(int64_t f, int64_t d) {
    return f * d;
}

int64_t power_watts(int64_t work, int64_t t) {
    return work / t;
}

int64_t pressure(int64_t f, int64_t area) {
    if (area == 0) return 0;
    return f / area;
}

int64_t density(int64_t mass, int64_t vol) {
    return mass / vol;
}

int64_t celsius_to_fahrenheit_x10(int64_t c) {
    return c * 18 + 320;
}

int64_t fahrenheit_to_celsius_x10(int64_t f) {
    return (f - 32) * 100 / 18;
}

int64_t celsius_to_kelvin_x10(int64_t c) {
    return c * 10 + 2732;
}

int64_t km_to_miles_scaled(int64_t km) {
    return km * 621 / 1000;
}

int64_t miles_to_km_scaled(int64_t mi) {
    return mi * 1609 / 1000;
}

int64_t mps_to_kmh(int64_t mps) {
    return mps * 36 / 10;
}

int64_t gravity_fall_dist(int64_t t) {
    return 5 * t * t;
}

int64_t potential_energy(int64_t m, int64_t h) {
    return m * 10 * h;
}

int main() {
    std::cout << "distance_uniform=" << distance_uniform(60, 2) << "\n";
    std::cout << "distance_accel=" << distance_accel(10, 4, 3) << "\n";
    std::cout << "final_velocity=" << final_velocity(5, 3, 4) << "\n";
    std::cout << "kinetic_energy_x2=" << kinetic_energy_x2(4, 5) << "\n";
    std::cout << "momentum=" << momentum(8, 3) << "\n";
    std::cout << "force=" << force(10, 9) << "\n";
    std::cout << "work=" << work(50, 4) << "\n";
    std::cout << "power_watts=" << power_watts(1000, 4) << "\n";
    std::cout << "pressure=" << pressure(200, 4) << "\n";
    std::cout << "density=" << density(1000, 8) << "\n";
    std::cout << "celsius_to_fahrenheit_x10=" << celsius_to_fahrenheit_x10(25) << "\n";
    std::cout << "fahrenheit_to_celsius_x10=" << fahrenheit_to_celsius_x10(77) << "\n";
    std::cout << "celsius_to_kelvin_x10=" << celsius_to_kelvin_x10(25) << "\n";
    std::cout << "km_to_miles_scaled=" << km_to_miles_scaled(100) << "\n";
    std::cout << "miles_to_km_scaled=" << miles_to_km_scaled(100) << "\n";
    std::cout << "mps_to_kmh=" << mps_to_kmh(10) << "\n";
    std::cout << "gravity_fall_dist=" << gravity_fall_dist(3) << "\n";
    std::cout << "potential_energy=" << potential_energy(2, 10) << "\n";
    return 0;
}
