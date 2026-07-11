// Cross-language hashing suite (C++). Deterministic integer hashes over an
// int64 byte array (values 0..255) plus its length. Arithmetic only — no
// bitwise operators, to stay algorithm-identical to Lullaby. Moduli and
// products are kept below 2^53 so JavaScript doubles stay exact too.
#include <cstdint>
#include <iostream>

int64_t djb2_hash(const int64_t *a, int64_t n) {
    int64_t m = 1000000007, h = 5381;
    for (int64_t i = 0; i < n; i++) h = (h * 33 + a[i]) % m;
    return h;
}

int64_t fnv1a_arithmetic(const int64_t *a, int64_t n) {
    int64_t m = 100000007, prime = 16777619;
    int64_t h = 2166136261LL % m;
    for (int64_t i = 0; i < n; i++) h = ((h + a[i]) * prime) % m;
    return h;
}

int64_t sum_hash(const int64_t *a, int64_t n) {
    int64_t total = 0;
    for (int64_t i = 0; i < n; i++) total += a[i];
    return total;
}

int64_t poly_hash(const int64_t *a, int64_t n, int64_t base, int64_t modulus) {
    int64_t h = 0;
    for (int64_t i = 0; i < n; i++) h = (h * base + a[i]) % modulus;
    return h;
}

int64_t rolling_hash(const int64_t *a, int64_t n, int64_t modulus) {
    int64_t h = 0;
    for (int64_t i = 0; i < n; i++) h = (h * 256 + a[i]) % modulus;
    return h;
}

int64_t sdbm_hash(const int64_t *a, int64_t n) {
    int64_t m = 1000000007, h = 0;
    for (int64_t i = 0; i < n; i++) h = (a[i] + h * 65599) % m;
    return h;
}

int64_t mod_sum_hash(const int64_t *a, int64_t n, int64_t m) {
    int64_t h = 0;
    for (int64_t i = 0; i < n; i++) h = (h + a[i]) % m;
    return h;
}

int64_t weighted_pos_hash(const int64_t *a, int64_t n) {
    int64_t m = 1000000007, h = 0;
    for (int64_t i = 0; i < n; i++) h = (h + (i + 1) * a[i]) % m;
    return h;
}

int64_t xor_free_checksum(const int64_t *a, int64_t n) {
    int64_t total = 0;
    for (int64_t i = 0; i < n; i++) total += a[i];
    return total % 256;
}

int64_t product_hash(const int64_t *a, int64_t n, int64_t m) {
    int64_t h = 1;
    for (int64_t i = 0; i < n; i++) h = (h * (a[i] + 1)) % m;
    return h;
}

int64_t rotate_hash(const int64_t *a, int64_t n) {
    int64_t m = 1073741824, half = 536870912, h = 0;
    for (int64_t i = 0; i < n; i++) {
        int64_t carry = h / half;
        h = (h * 2) % m + carry;
        h = (h + a[i]) % m;
    }
    return h;
}

int64_t pearson_like(const int64_t *a, int64_t n) {
    int64_t t[16] = { 7, 3, 11, 15, 0, 9, 5, 13, 1, 14, 6, 10, 2, 12, 4, 8 };
    int64_t h = 0;
    for (int64_t i = 0; i < n; i++) h = t[(h + a[i]) % 16];
    return h;
}

int64_t length_hash(const int64_t *a, int64_t n) {
    int64_t m = 1000000007;
    if (n <= 0) return 0;
    return (n * 2654435761LL + a[0]) % m;
}

int64_t first_last_hash(const int64_t *a, int64_t n) {
    if (n <= 0) return 0;
    return a[0] * 257 + a[n - 1];
}

int64_t midpoint_hash(const int64_t *a, int64_t n) {
    if (n <= 0) return 0;
    return a[n / 2] * 33 + n;
}

int64_t digit_hash(int64_t x) {
    if (x < 0) x = -x;
    if (x == 0) return 0;
    int64_t m = 1000000007, h = 7;
    while (x > 0) {
        h = (h * 31 + x % 10) % m;
        x /= 10;
    }
    return h;
}

int64_t pair_hash(int64_t a, int64_t b) {
    int64_t s = a + b;
    return s * (s + 1) / 2 + b;
}

int main() {
    int64_t data[] = { 72, 101, 108, 108, 111 };
    std::cout << "djb2_hash=" << djb2_hash(data, 5) << "\n";
    std::cout << "fnv1a_arithmetic=" << fnv1a_arithmetic(data, 5) << "\n";
    std::cout << "sum_hash=" << sum_hash(data, 5) << "\n";
    std::cout << "poly_hash=" << poly_hash(data, 5, 31, 1000000007) << "\n";
    std::cout << "rolling_hash=" << rolling_hash(data, 5, 1000000007) << "\n";
    std::cout << "sdbm_hash=" << sdbm_hash(data, 5) << "\n";
    std::cout << "mod_sum_hash=" << mod_sum_hash(data, 5, 97) << "\n";
    std::cout << "weighted_pos_hash=" << weighted_pos_hash(data, 5) << "\n";
    std::cout << "xor_free_checksum=" << xor_free_checksum(data, 5) << "\n";
    std::cout << "product_hash=" << product_hash(data, 5, 1000000007) << "\n";
    std::cout << "rotate_hash=" << rotate_hash(data, 5) << "\n";
    std::cout << "pearson_like=" << pearson_like(data, 5) << "\n";
    std::cout << "length_hash=" << length_hash(data, 5) << "\n";
    std::cout << "first_last_hash=" << first_last_hash(data, 5) << "\n";
    std::cout << "midpoint_hash=" << midpoint_hash(data, 5) << "\n";
    std::cout << "digit_hash=" << digit_hash(1234) << "\n";
    std::cout << "pair_hash=" << pair_hash(17, 42) << "\n";
    return 0;
}
