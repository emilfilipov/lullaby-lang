// Cross-language validation suite (C). Real-world validators mirroring
// ../validation/lullaby.lby: checksums, date rules, character classes, ranges.
// Arrays are const long long* + length; 1-char args are const char* (c[0]).

#include <assert.h>
#include <stdio.h>
#include <string.h>

typedef long long i64;

i64 luhn_check(const i64 *digits, i64 n) {
    i64 sum = 0;
    for (i64 i = 0; i < n; i++) {
        i64 d = digits[i];
        i64 pos = n - 1 - i;
        if (pos % 2 == 1) {
            d *= 2;
            if (d > 9) d -= 9;
        }
        sum += d;
    }
    return sum % 10 == 0 ? 1 : 0;
}

i64 is_valid_isbn10(const i64 *digits, i64 n) {
    if (n != 10) return 0;
    i64 sum = 0;
    for (i64 i = 0; i < 10; i++) {
        sum += (10 - i) * digits[i];
    }
    return sum % 11 == 0 ? 1 : 0;
}

i64 in_range(i64 x, i64 lo, i64 hi) {
    return x >= lo && x <= hi ? 1 : 0;
}

i64 is_leap_year(i64 y) {
    if (y % 400 == 0) return 1;
    if (y % 100 == 0) return 0;
    return y % 4 == 0 ? 1 : 0;
}

i64 is_valid_month(i64 m) {
    return m >= 1 && m <= 12 ? 1 : 0;
}

i64 days_in_month(i64 y, i64 m) {
    if (m == 2) return is_leap_year(y) ? 29 : 28;
    if (m == 4 || m == 6 || m == 9 || m == 11) return 30;
    if (m >= 1 && m <= 12) return 31;
    return 0;
}

i64 is_valid_day(i64 y, i64 m, i64 d) {
    if (!is_valid_month(m)) return 0;
    if (d < 1 || d > days_in_month(y, m)) return 0;
    return 1;
}

i64 is_ascii_digit(const char *c) {
    return c[0] >= '0' && c[0] <= '9' ? 1 : 0;
}

i64 is_ascii_alpha(const char *c) {
    return (c[0] >= 'A' && c[0] <= 'Z') || (c[0] >= 'a' && c[0] <= 'z') ? 1 : 0;
}

i64 all_digits(const char *s) {
    size_t n = strlen(s);
    if (n == 0) return 0;
    for (size_t i = 0; i < n; i++) {
        if (s[i] < '0' || s[i] > '9') return 0;
    }
    return 1;
}

i64 password_score(const char *s) {
    size_t n = strlen(s);
    i64 score = 0;
    if (n >= 8) score++;
    i64 has_digit = 0, has_lower = 0, has_upper = 0;
    for (size_t i = 0; i < n; i++) {
        char c = s[i];
        if (c >= '0' && c <= '9') has_digit = 1;
        else if (c >= 'a' && c <= 'z') has_lower = 1;
        else if (c >= 'A' && c <= 'Z') has_upper = 1;
    }
    return score + has_digit + has_lower + has_upper;
}

i64 is_hex_string(const char *s) {
    size_t n = strlen(s);
    if (n == 0) return 0;
    for (size_t i = 0; i < n; i++) {
        char c = s[i];
        int ok = (c >= '0' && c <= '9') || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F');
        if (!ok) return 0;
    }
    return 1;
}

i64 checksum_mod10(const i64 *digits, i64 n) {
    i64 sum = 0;
    for (i64 i = 0; i < n; i++) sum += digits[i];
    return sum % 10;
}

i64 valid_percentage(i64 x) {
    return x >= 0 && x <= 100 ? 1 : 0;
}

i64 is_valid_rgb(i64 r, i64 g, i64 b) {
    return r >= 0 && r <= 255 && g >= 0 && g <= 255 && b >= 0 && b <= 255 ? 1 : 0;
}

i64 even_parity(const i64 *bits, i64 n) {
    i64 ones = 0;
    for (i64 i = 0; i < n; i++) {
        if (bits[i] == 1) ones++;
    }
    return ones % 2 == 0 ? 1 : 0;
}

int main(void) {
    i64 card[] = {7, 9, 9, 2, 7, 3, 9, 8, 7, 1, 3};
    i64 isbn[] = {0, 3, 0, 6, 4, 0, 6, 1, 5, 2};
    i64 bits[] = {1, 0, 1, 1, 0, 0};
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
    printf("ok\n");
    return 0;
}
