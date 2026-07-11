/* Cross-language encoding suite (C). Checksums, ciphers, bit tricks over
   int64 byte arrays (values 0..255). Arithmetic only — no bitwise operators,
   to stay algorithm-identical to Lullaby. */
#include <stdio.h>
#include <stdint.h>

int64_t sum_bytes(const int64_t *a, int64_t n) {
    int64_t total = 0;
    for (int64_t i = 0; i < n; i++) total += a[i];
    return total;
}

int64_t add_checksum_mod256(const int64_t *a, int64_t n) {
    int64_t total = 0;
    for (int64_t i = 0; i < n; i++) total += a[i];
    return total % 256;
}

int64_t fletcher16(const int64_t *a, int64_t n) {
    int64_t sum1 = 0, sum2 = 0;
    for (int64_t i = 0; i < n; i++) {
        sum1 = (sum1 + a[i]) % 255;
        sum2 = (sum2 + sum1) % 255;
    }
    return sum2 * 256 + sum1;
}

int64_t adler32_small(const int64_t *a, int64_t n) {
    int64_t s1 = 1, s2 = 0;
    for (int64_t i = 0; i < n; i++) {
        s1 = (s1 + a[i]) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    return s2 * 65536 + s1;
}

int64_t caesar_encrypt_val(int64_t c, int64_t k) {
    if (c >= 97 && c <= 122) return 97 + (c - 97 + k % 26) % 26;
    return c;
}

int64_t caesar_decrypt_val(int64_t c, int64_t k) {
    if (c >= 97 && c <= 122) return 97 + (c - 97 + 26 - k % 26) % 26;
    return c;
}

int64_t rot13_val(int64_t c) {
    if (c >= 97 && c <= 122) return 97 + (c - 97 + 13) % 26;
    if (c >= 65 && c <= 90) return 65 + (c - 65 + 13) % 26;
    return c;
}

int64_t count_set_bits(int64_t x) {
    int64_t count = 0;
    while (x > 0) {
        count += x % 2;
        x /= 2;
    }
    return count;
}

int64_t to_binary_length(int64_t x) {
    if (x == 0) return 1;
    int64_t len = 0;
    while (x > 0) {
        len++;
        x /= 2;
    }
    return len;
}

int64_t hex_digit_value(int64_t c) {
    if (c >= 48 && c <= 57) return c - 48;
    if (c >= 97 && c <= 102) return c - 97 + 10;
    return -1;
}

int64_t nibble_to_hex_code(int64_t v) {
    if (v < 10) return 48 + v;
    return 97 + v - 10;
}

int64_t luhn_from_array(const int64_t *a, int64_t n) {
    int64_t sum = 0;
    for (int64_t i = 0; i < n; i++) {
        int64_t d = a[n - 1 - i];
        if (i % 2 == 1) {
            d = d * 2;
            if (d > 9) d = d - 9;
        }
        sum += d;
    }
    return sum % 10 == 0 ? 1 : 0;
}

int64_t parity_bit(const int64_t *a, int64_t n) {
    int64_t ones = 0;
    for (int64_t i = 0; i < n; i++) ones += a[i];
    return ones % 2;
}

int64_t crc8_simple(const int64_t *a, int64_t n) {
    int64_t crc = 0;
    for (int64_t i = 0; i < n; i++) {
        crc = (crc + a[i]) % 256;
        crc = (crc * 2) % 256 + crc / 128;
    }
    return crc;
}

int64_t digit_product(int64_t x) {
    if (x < 0) x = -x;
    if (x == 0) return 0;
    int64_t p = 1;
    while (x > 0) {
        p = p * (x % 10);
        x /= 10;
    }
    return p;
}

int main(void) {
    int64_t data[] = { 72, 101, 108, 108, 111 };
    printf("sum_bytes=%lld\n", (long long)sum_bytes(data, 5));
    printf("add_checksum_mod256=%lld\n", (long long)add_checksum_mod256(data, 5));
    printf("fletcher16=%lld\n", (long long)fletcher16(data, 5));
    printf("adler32_small=%lld\n", (long long)adler32_small(data, 5));
    printf("caesar_encrypt_val=%lld\n", (long long)caesar_encrypt_val(104, 3));
    printf("caesar_decrypt_val=%lld\n", (long long)caesar_decrypt_val(107, 3));
    printf("rot13_val=%lld\n", (long long)rot13_val(97));
    printf("count_set_bits=%lld\n", (long long)count_set_bits(181));
    printf("to_binary_length=%lld\n", (long long)to_binary_length(181));
    printf("hex_digit_value=%lld\n", (long long)hex_digit_value(102));
    printf("nibble_to_hex_code=%lld\n", (long long)nibble_to_hex_code(12));
    int64_t card[] = { 7, 9, 9, 2, 7, 3, 9, 8, 7, 1, 3 };
    printf("luhn_from_array=%lld\n", (long long)luhn_from_array(card, 11));
    int64_t bits[] = { 1, 0, 1, 1, 0 };
    printf("parity_bit=%lld\n", (long long)parity_bit(bits, 5));
    printf("crc8_simple=%lld\n", (long long)crc8_simple(data, 5));
    printf("digit_product=%lld\n", (long long)digit_product(1234));
    return 0;
}
