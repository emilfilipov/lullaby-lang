/* Cross-language units suite (C). Unit/quantity conversions in integers. */
#include <stdio.h>
#include <stdint.h>

int64_t minutes_to_seconds(int64_t m) {
    return m * 60;
}

int64_t hours_to_minutes(int64_t h) {
    return h * 60;
}

int64_t days_to_hours(int64_t d) {
    return d * 24;
}

int64_t weeks_to_days(int64_t w) {
    return w * 7;
}

int64_t bytes_to_kib(int64_t b) {
    return b / 1024;
}

int64_t kib_to_bytes(int64_t k) {
    return k * 1024;
}

int64_t mib_to_bytes(int64_t m) {
    return m * 1024 * 1024;
}

int64_t meters_to_cm(int64_t m) {
    return m * 100;
}

int64_t cm_to_mm(int64_t c) {
    return c * 10;
}

int64_t km_to_meters(int64_t k) {
    return k * 1000;
}

int64_t feet_to_inches(int64_t f) {
    return f * 12;
}

int64_t inches_to_cm_x10(int64_t i) {
    return i * 254 / 10;
}

int64_t pounds_to_ounces(int64_t p) {
    return p * 16;
}

int64_t gallons_to_pints(int64_t g) {
    return g * 8;
}

int64_t liters_to_ml(int64_t l) {
    return l * 1000;
}

int64_t degrees_to_gradians(int64_t d) {
    return d * 10 / 9;
}

int64_t radians_milli_to_degrees(int64_t mr) {
    return mr * 180 / 3142;
}

int64_t seconds_to_hms_hours(int64_t s) {
    return s / 3600;
}

int main(void) {
    printf("minutes_to_seconds=%lld\n", (long long)minutes_to_seconds(5));
    printf("hours_to_minutes=%lld\n", (long long)hours_to_minutes(3));
    printf("days_to_hours=%lld\n", (long long)days_to_hours(2));
    printf("weeks_to_days=%lld\n", (long long)weeks_to_days(3));
    printf("bytes_to_kib=%lld\n", (long long)bytes_to_kib(4096));
    printf("kib_to_bytes=%lld\n", (long long)kib_to_bytes(4));
    printf("mib_to_bytes=%lld\n", (long long)mib_to_bytes(2));
    printf("meters_to_cm=%lld\n", (long long)meters_to_cm(7));
    printf("cm_to_mm=%lld\n", (long long)cm_to_mm(12));
    printf("km_to_meters=%lld\n", (long long)km_to_meters(5));
    printf("feet_to_inches=%lld\n", (long long)feet_to_inches(6));
    printf("inches_to_cm_x10=%lld\n", (long long)inches_to_cm_x10(10));
    printf("pounds_to_ounces=%lld\n", (long long)pounds_to_ounces(3));
    printf("gallons_to_pints=%lld\n", (long long)gallons_to_pints(2));
    printf("liters_to_ml=%lld\n", (long long)liters_to_ml(3));
    printf("degrees_to_gradians=%lld\n", (long long)degrees_to_gradians(90));
    printf("radians_milli_to_degrees=%lld\n", (long long)radians_milli_to_degrees(3142));
    printf("seconds_to_hms_hours=%lld\n", (long long)seconds_to_hms_hours(7325));
    return 0;
}
