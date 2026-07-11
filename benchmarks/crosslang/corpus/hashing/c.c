/* Cross-language hashing suite (C). Deterministic integer hashes over an
   int64 byte array (values 0..255) plus its length. Arithmetic only — no
   bitwise operators, to stay algorithm-identical to Lullaby. Moduli and
   products are kept below 2^53 so JavaScript doubles stay exact too. */
#include <stdio.h>
#include <stdint.h>

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

int main(void) {
    int64_t data[] = { 72, 101, 108, 108, 111 };
    printf("djb2_hash=%lld\n", (long long)djb2_hash(data, 5));
    printf("fnv1a_arithmetic=%lld\n", (long long)fnv1a_arithmetic(data, 5));
    printf("sum_hash=%lld\n", (long long)sum_hash(data, 5));
    printf("poly_hash=%lld\n", (long long)poly_hash(data, 5, 31, 1000000007));
    printf("rolling_hash=%lld\n", (long long)rolling_hash(data, 5, 1000000007));
    printf("sdbm_hash=%lld\n", (long long)sdbm_hash(data, 5));
    printf("mod_sum_hash=%lld\n", (long long)mod_sum_hash(data, 5, 97));
    printf("weighted_pos_hash=%lld\n", (long long)weighted_pos_hash(data, 5));
    printf("xor_free_checksum=%lld\n", (long long)xor_free_checksum(data, 5));
    printf("product_hash=%lld\n", (long long)product_hash(data, 5, 1000000007));
    printf("rotate_hash=%lld\n", (long long)rotate_hash(data, 5));
    printf("pearson_like=%lld\n", (long long)pearson_like(data, 5));
    printf("length_hash=%lld\n", (long long)length_hash(data, 5));
    printf("first_last_hash=%lld\n", (long long)first_last_hash(data, 5));
    printf("midpoint_hash=%lld\n", (long long)midpoint_hash(data, 5));
    printf("digit_hash=%lld\n", (long long)digit_hash(1234));
    printf("pair_hash=%lld\n", (long long)pair_hash(17, 42));
    return 0;
}
