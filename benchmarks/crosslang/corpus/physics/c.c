/* Cross-language physics suite (C). Integer physics & unit conversions.
   All math is scaled-integer (no floats): temperatures are tenths (x10),
   distances/speeds use fixed scale factors, energy uses g = 10 m/s^2. */
#include <stdio.h>
#include <stdint.h>

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

int main(void) {
    printf("distance_uniform=%lld\n", (long long)distance_uniform(60, 2));
    printf("distance_accel=%lld\n", (long long)distance_accel(10, 4, 3));
    printf("final_velocity=%lld\n", (long long)final_velocity(5, 3, 4));
    printf("kinetic_energy_x2=%lld\n", (long long)kinetic_energy_x2(4, 5));
    printf("momentum=%lld\n", (long long)momentum(8, 3));
    printf("force=%lld\n", (long long)force(10, 9));
    printf("work=%lld\n", (long long)work(50, 4));
    printf("power_watts=%lld\n", (long long)power_watts(1000, 4));
    printf("pressure=%lld\n", (long long)pressure(200, 4));
    printf("density=%lld\n", (long long)density(1000, 8));
    printf("celsius_to_fahrenheit_x10=%lld\n", (long long)celsius_to_fahrenheit_x10(25));
    printf("fahrenheit_to_celsius_x10=%lld\n", (long long)fahrenheit_to_celsius_x10(77));
    printf("celsius_to_kelvin_x10=%lld\n", (long long)celsius_to_kelvin_x10(25));
    printf("km_to_miles_scaled=%lld\n", (long long)km_to_miles_scaled(100));
    printf("miles_to_km_scaled=%lld\n", (long long)miles_to_km_scaled(100));
    printf("mps_to_kmh=%lld\n", (long long)mps_to_kmh(10));
    printf("gravity_fall_dist=%lld\n", (long long)gravity_fall_dist(3));
    printf("potential_energy=%lld\n", (long long)potential_energy(2, 10));
    return 0;
}
