// Cross-language validation suite (C++). Mirrors ../validation/lullaby.lby.
// Arrays are const long long* + length (uniform with the C baseline); string
// args use std::string. All predicates return long long (1/0).
#include <cassert>
#include <iostream>
#include <string>

long long luhn_check(const long long *digits, long long n) {
    long long sum = 0;
    for (long long i = 0; i < n; i++) {
        long long d = digits[i];
        long long pos = n - 1 - i;
        if (pos % 2 == 1) {
            d *= 2;
            if (d > 9) d -= 9;
        }
        sum += d;
    }
    return sum % 10 == 0 ? 1 : 0;
}

long long is_valid_isbn10(const long long *digits, long long n) {
    if (n != 10) return 0;
    long long sum = 0;
    for (long long i = 0; i < 10; i++) {
        sum += (10 - i) * digits[i];
    }
    return sum % 11 == 0 ? 1 : 0;
}

long long in_range(long long x, long long lo, long long hi) {
    return x >= lo && x <= hi ? 1 : 0;
}

long long is_leap_year(long long y) {
    if (y % 400 == 0) return 1;
    if (y % 100 == 0) return 0;
    return y % 4 == 0 ? 1 : 0;
}

long long is_valid_month(long long m) {
    return m >= 1 && m <= 12 ? 1 : 0;
}

long long days_in_month(long long y, long long m) {
    if (m == 2) return is_leap_year(y) ? 29 : 28;
    if (m == 4 || m == 6 || m == 9 || m == 11) return 30;
    if (m >= 1 && m <= 12) return 31;
    return 0;
}

long long is_valid_day(long long y, long long m, long long d) {
    if (!is_valid_month(m)) return 0;
    if (d < 1 || d > days_in_month(y, m)) return 0;
    return 1;
}

long long is_ascii_digit(const std::string &c) {
    return c[0] >= '0' && c[0] <= '9' ? 1 : 0;
}

long long is_ascii_alpha(const std::string &c) {
    return (c[0] >= 'A' && c[0] <= 'Z') || (c[0] >= 'a' && c[0] <= 'z') ? 1 : 0;
}

long long all_digits(const std::string &s) {
    if (s.empty()) return 0;
    for (char c : s) {
        if (c < '0' || c > '9') return 0;
    }
    return 1;
}

long long password_score(const std::string &s) {
    long long score = 0;
    if (s.size() >= 8) score++;
    long long has_digit = 0, has_lower = 0, has_upper = 0;
    for (char c : s) {
        if (c >= '0' && c <= '9') has_digit = 1;
        else if (c >= 'a' && c <= 'z') has_lower = 1;
        else if (c >= 'A' && c <= 'Z') has_upper = 1;
    }
    return score + has_digit + has_lower + has_upper;
}

long long is_hex_string(const std::string &s) {
    if (s.empty()) return 0;
    for (char c : s) {
        bool ok = (c >= '0' && c <= '9') || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F');
        if (!ok) return 0;
    }
    return 1;
}

long long checksum_mod10(const long long *digits, long long n) {
    long long sum = 0;
    for (long long i = 0; i < n; i++) sum += digits[i];
    return sum % 10;
}

long long valid_percentage(long long x) {
    return x >= 0 && x <= 100 ? 1 : 0;
}

long long is_valid_rgb(long long r, long long g, long long b) {
    return r >= 0 && r <= 255 && g >= 0 && g <= 255 && b >= 0 && b <= 255 ? 1 : 0;
}

long long even_parity(const long long *bits, long long n) {
    long long ones = 0;
    for (long long i = 0; i < n; i++) {
        if (bits[i] == 1) ones++;
    }
    return ones % 2 == 0 ? 1 : 0;
}

int main() {
    const long long card[] = {7, 9, 9, 2, 7, 3, 9, 8, 7, 1, 3};
    const long long isbn[] = {0, 3, 0, 6, 4, 0, 6, 1, 5, 2};
    const long long bits[] = {1, 0, 1, 1, 0, 0};
    assert(luhn_check(card, 11) == 1);
    assert(is_valid_isbn10(isbn, 10) == 1);
    assert(in_range(5, 1, 10) == 1);
    assert(is_leap_year(2000) == 1);
    assert(is_leap_year(1900) == 0);
    assert(is_valid_month(13) == 0);
    assert(is_valid_day(2021, 2, 29) == 0);
    assert(is_valid_day(2020, 2, 29) == 1);
    assert(is_ascii_digit("7") == 1);
    assert(is_ascii_alpha("Q") == 1);
    assert(all_digits("12345") == 1);
    assert(password_score("Abcdef12") == 4);
    assert(is_hex_string("1aF") == 1);
    assert(checksum_mod10(card, 11) == 5);
    assert(valid_percentage(101) == 0);
    assert(is_valid_rgb(255, 128, 0) == 1);
    assert(even_parity(bits, 6) == 0);
    std::cout << "ok" << std::endl;
    return 0;
}
