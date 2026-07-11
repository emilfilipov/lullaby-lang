// Cross-language units suite (C++). Unit/quantity conversions in integers.
#include <cstdint>
#include <iostream>

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

int main() {
    std::cout << "minutes_to_seconds=" << minutes_to_seconds(5) << "\n";
    std::cout << "hours_to_minutes=" << hours_to_minutes(3) << "\n";
    std::cout << "days_to_hours=" << days_to_hours(2) << "\n";
    std::cout << "weeks_to_days=" << weeks_to_days(3) << "\n";
    std::cout << "bytes_to_kib=" << bytes_to_kib(4096) << "\n";
    std::cout << "kib_to_bytes=" << kib_to_bytes(4) << "\n";
    std::cout << "mib_to_bytes=" << mib_to_bytes(2) << "\n";
    std::cout << "meters_to_cm=" << meters_to_cm(7) << "\n";
    std::cout << "cm_to_mm=" << cm_to_mm(12) << "\n";
    std::cout << "km_to_meters=" << km_to_meters(5) << "\n";
    std::cout << "feet_to_inches=" << feet_to_inches(6) << "\n";
    std::cout << "inches_to_cm_x10=" << inches_to_cm_x10(10) << "\n";
    std::cout << "pounds_to_ounces=" << pounds_to_ounces(3) << "\n";
    std::cout << "gallons_to_pints=" << gallons_to_pints(2) << "\n";
    std::cout << "liters_to_ml=" << liters_to_ml(3) << "\n";
    std::cout << "degrees_to_gradians=" << degrees_to_gradians(90) << "\n";
    std::cout << "radians_milli_to_degrees=" << radians_milli_to_degrees(3142) << "\n";
    std::cout << "seconds_to_hms_hours=" << seconds_to_hms_hours(7325) << "\n";
    return 0;
}
