/* Cross-language matrix suite (C). Dense matrix operations over a flat
   row-major i64 array: an n*n matrix is passed as a pointer plus its order n,
   with element (r, c) stored at m[r * n + c]. */
#include <stdio.h>
#include <stdint.h>

int64_t diagonal_sum_main(const int64_t *m, int64_t n) {
    int64_t sum = 0;
    for (int64_t i = 0; i < n; i++) sum += m[i * n + i];
    return sum;
}

int64_t trace(const int64_t *m, int64_t n) {
    return diagonal_sum_main(m, n);
}

int64_t diagonal_sum_anti(const int64_t *m, int64_t n) {
    int64_t sum = 0;
    for (int64_t i = 0; i < n; i++) sum += m[i * n + (n - 1 - i)];
    return sum;
}

int64_t row_sum(const int64_t *m, int64_t n, int64_t r) {
    int64_t sum = 0;
    for (int64_t c = 0; c < n; c++) sum += m[r * n + c];
    return sum;
}

int64_t col_sum(const int64_t *m, int64_t n, int64_t c) {
    int64_t sum = 0;
    for (int64_t r = 0; r < n; r++) sum += m[r * n + c];
    return sum;
}

int64_t row_sum_max(const int64_t *m, int64_t n) {
    int64_t best = row_sum(m, n, 0);
    for (int64_t r = 1; r < n; r++) {
        int64_t s = row_sum(m, n, r);
        if (s > best) best = s;
    }
    return best;
}

int64_t col_sum_max(const int64_t *m, int64_t n) {
    int64_t best = col_sum(m, n, 0);
    for (int64_t c = 1; c < n; c++) {
        int64_t s = col_sum(m, n, c);
        if (s > best) best = s;
    }
    return best;
}

int64_t is_symmetric(const int64_t *m, int64_t n) {
    for (int64_t i = 0; i < n; i++)
        for (int64_t j = 0; j < n; j++)
            if (m[i * n + j] != m[j * n + i]) return 0;
    return 1;
}

int64_t transpose_checksum(const int64_t *m, int64_t n) {
    int64_t sum = 0;
    for (int64_t r = 0; r < n; r++)
        for (int64_t c = 0; c < n; c++) {
            int64_t i = r * n + c;
            sum += i * m[c * n + r];
        }
    return sum;
}

int64_t matrix_add_checksum(const int64_t *a, const int64_t *b, int64_t n) {
    int64_t sum = 0;
    for (int64_t i = 0; i < n * n; i++) sum += i * (a[i] + b[i]);
    return sum;
}

int64_t scalar_mul_checksum(const int64_t *m, int64_t n, int64_t k) {
    int64_t sum = 0;
    for (int64_t i = 0; i < n * n; i++) sum += i * (m[i] * k);
    return sum;
}

int64_t is_identity(const int64_t *m, int64_t n) {
    for (int64_t i = 0; i < n; i++)
        for (int64_t j = 0; j < n; j++) {
            if (i == j) { if (m[i * n + j] != 1) return 0; }
            else { if (m[i * n + j] != 0) return 0; }
        }
    return 1;
}

int64_t matrix_mul_trace(const int64_t *a, const int64_t *b, int64_t n) {
    int64_t sum = 0;
    for (int64_t i = 0; i < n; i++)
        for (int64_t k = 0; k < n; k++)
            sum += a[i * n + k] * b[k * n + i];
    return sum;
}

int64_t main_diag_product(const int64_t *m, int64_t n) {
    int64_t product = 1;
    for (int64_t i = 0; i < n; i++) product *= m[i * n + i];
    return product;
}

int64_t max_element(const int64_t *m, int64_t n) {
    int64_t best = m[0];
    for (int64_t i = 1; i < n * n; i++) if (m[i] > best) best = m[i];
    return best;
}

int64_t min_element(const int64_t *m, int64_t n) {
    int64_t best = m[0];
    for (int64_t i = 1; i < n * n; i++) if (m[i] < best) best = m[i];
    return best;
}

int64_t determinant_2x2(const int64_t *m) {
    return m[0] * m[3] - m[1] * m[2];
}

int64_t determinant_3x3(const int64_t *m) {
    return m[0] * (m[4] * m[8] - m[5] * m[7])
         - m[1] * (m[3] * m[8] - m[5] * m[6])
         + m[2] * (m[3] * m[7] - m[4] * m[6]);
}

int main(void) {
    int64_t m[9] = { 1, 2, 3, 4, 5, 6, 7, 8, 9 };
    int64_t sym[9] = { 1, 2, 3, 2, 5, 6, 3, 6, 9 };
    int64_t id[9] = { 1, 0, 0, 0, 1, 0, 0, 0, 1 };
    int64_t b[9] = { 9, 8, 7, 6, 5, 4, 3, 2, 1 };
    int64_t d2[4] = { 1, 2, 3, 4 };
    printf("trace=%lld\n", (long long)trace(m, 3));
    printf("diagonal_sum_main=%lld\n", (long long)diagonal_sum_main(m, 3));
    printf("diagonal_sum_anti=%lld\n", (long long)diagonal_sum_anti(m, 3));
    printf("row_sum=%lld\n", (long long)row_sum(m, 3, 1));
    printf("col_sum=%lld\n", (long long)col_sum(m, 3, 2));
    printf("row_sum_max=%lld\n", (long long)row_sum_max(m, 3));
    printf("col_sum_max=%lld\n", (long long)col_sum_max(m, 3));
    printf("is_symmetric=%lld\n", (long long)is_symmetric(sym, 3));
    printf("transpose_checksum=%lld\n", (long long)transpose_checksum(m, 3));
    printf("matrix_add_checksum=%lld\n", (long long)matrix_add_checksum(m, b, 3));
    printf("scalar_mul_checksum=%lld\n", (long long)scalar_mul_checksum(m, 3, 2));
    printf("is_identity=%lld\n", (long long)is_identity(id, 3));
    printf("matrix_mul_trace=%lld\n", (long long)matrix_mul_trace(m, b, 3));
    printf("main_diag_product=%lld\n", (long long)main_diag_product(m, 3));
    printf("max_element=%lld\n", (long long)max_element(m, 3));
    printf("min_element=%lld\n", (long long)min_element(m, 3));
    printf("determinant_2x2=%lld\n", (long long)determinant_2x2(d2));
    printf("determinant_3x3=%lld\n", (long long)determinant_3x3(m));
    return 0;
}
