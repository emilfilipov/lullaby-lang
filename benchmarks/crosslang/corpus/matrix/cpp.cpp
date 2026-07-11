// Cross-language matrix suite (C++). Dense matrix operations over a flat
// row-major i64 array: an n*n matrix is passed as a pointer plus its order n,
// with element (r, c) stored at m[r * n + c].
#include <cstdint>
#include <iostream>

std::int64_t diagonal_sum_main(const std::int64_t *m, std::int64_t n) {
    std::int64_t sum = 0;
    for (std::int64_t i = 0; i < n; i++) sum += m[i * n + i];
    return sum;
}

std::int64_t trace(const std::int64_t *m, std::int64_t n) {
    return diagonal_sum_main(m, n);
}

std::int64_t diagonal_sum_anti(const std::int64_t *m, std::int64_t n) {
    std::int64_t sum = 0;
    for (std::int64_t i = 0; i < n; i++) sum += m[i * n + (n - 1 - i)];
    return sum;
}

std::int64_t row_sum(const std::int64_t *m, std::int64_t n, std::int64_t r) {
    std::int64_t sum = 0;
    for (std::int64_t c = 0; c < n; c++) sum += m[r * n + c];
    return sum;
}

std::int64_t col_sum(const std::int64_t *m, std::int64_t n, std::int64_t c) {
    std::int64_t sum = 0;
    for (std::int64_t r = 0; r < n; r++) sum += m[r * n + c];
    return sum;
}

std::int64_t row_sum_max(const std::int64_t *m, std::int64_t n) {
    std::int64_t best = row_sum(m, n, 0);
    for (std::int64_t r = 1; r < n; r++) {
        std::int64_t s = row_sum(m, n, r);
        if (s > best) best = s;
    }
    return best;
}

std::int64_t col_sum_max(const std::int64_t *m, std::int64_t n) {
    std::int64_t best = col_sum(m, n, 0);
    for (std::int64_t c = 1; c < n; c++) {
        std::int64_t s = col_sum(m, n, c);
        if (s > best) best = s;
    }
    return best;
}

std::int64_t is_symmetric(const std::int64_t *m, std::int64_t n) {
    for (std::int64_t i = 0; i < n; i++)
        for (std::int64_t j = 0; j < n; j++)
            if (m[i * n + j] != m[j * n + i]) return 0;
    return 1;
}

std::int64_t transpose_checksum(const std::int64_t *m, std::int64_t n) {
    std::int64_t sum = 0;
    for (std::int64_t r = 0; r < n; r++)
        for (std::int64_t c = 0; c < n; c++) {
            std::int64_t i = r * n + c;
            sum += i * m[c * n + r];
        }
    return sum;
}

std::int64_t matrix_add_checksum(const std::int64_t *a, const std::int64_t *b, std::int64_t n) {
    std::int64_t sum = 0;
    for (std::int64_t i = 0; i < n * n; i++) sum += i * (a[i] + b[i]);
    return sum;
}

std::int64_t scalar_mul_checksum(const std::int64_t *m, std::int64_t n, std::int64_t k) {
    std::int64_t sum = 0;
    for (std::int64_t i = 0; i < n * n; i++) sum += i * (m[i] * k);
    return sum;
}

std::int64_t is_identity(const std::int64_t *m, std::int64_t n) {
    for (std::int64_t i = 0; i < n; i++)
        for (std::int64_t j = 0; j < n; j++) {
            if (i == j) { if (m[i * n + j] != 1) return 0; }
            else { if (m[i * n + j] != 0) return 0; }
        }
    return 1;
}

std::int64_t matrix_mul_trace(const std::int64_t *a, const std::int64_t *b, std::int64_t n) {
    std::int64_t sum = 0;
    for (std::int64_t i = 0; i < n; i++)
        for (std::int64_t k = 0; k < n; k++)
            sum += a[i * n + k] * b[k * n + i];
    return sum;
}

std::int64_t main_diag_product(const std::int64_t *m, std::int64_t n) {
    std::int64_t product = 1;
    for (std::int64_t i = 0; i < n; i++) product *= m[i * n + i];
    return product;
}

std::int64_t max_element(const std::int64_t *m, std::int64_t n) {
    std::int64_t best = m[0];
    for (std::int64_t i = 1; i < n * n; i++) if (m[i] > best) best = m[i];
    return best;
}

std::int64_t min_element(const std::int64_t *m, std::int64_t n) {
    std::int64_t best = m[0];
    for (std::int64_t i = 1; i < n * n; i++) if (m[i] < best) best = m[i];
    return best;
}

std::int64_t determinant_2x2(const std::int64_t *m) {
    return m[0] * m[3] - m[1] * m[2];
}

std::int64_t determinant_3x3(const std::int64_t *m) {
    return m[0] * (m[4] * m[8] - m[5] * m[7])
         - m[1] * (m[3] * m[8] - m[5] * m[6])
         + m[2] * (m[3] * m[7] - m[4] * m[6]);
}

int main() {
    std::int64_t m[9] = { 1, 2, 3, 4, 5, 6, 7, 8, 9 };
    std::int64_t sym[9] = { 1, 2, 3, 2, 5, 6, 3, 6, 9 };
    std::int64_t id[9] = { 1, 0, 0, 0, 1, 0, 0, 0, 1 };
    std::int64_t b[9] = { 9, 8, 7, 6, 5, 4, 3, 2, 1 };
    std::int64_t d2[4] = { 1, 2, 3, 4 };
    std::cout << "trace=" << trace(m, 3) << "\n";
    std::cout << "diagonal_sum_main=" << diagonal_sum_main(m, 3) << "\n";
    std::cout << "diagonal_sum_anti=" << diagonal_sum_anti(m, 3) << "\n";
    std::cout << "row_sum=" << row_sum(m, 3, 1) << "\n";
    std::cout << "col_sum=" << col_sum(m, 3, 2) << "\n";
    std::cout << "row_sum_max=" << row_sum_max(m, 3) << "\n";
    std::cout << "col_sum_max=" << col_sum_max(m, 3) << "\n";
    std::cout << "is_symmetric=" << is_symmetric(sym, 3) << "\n";
    std::cout << "transpose_checksum=" << transpose_checksum(m, 3) << "\n";
    std::cout << "matrix_add_checksum=" << matrix_add_checksum(m, b, 3) << "\n";
    std::cout << "scalar_mul_checksum=" << scalar_mul_checksum(m, 3, 2) << "\n";
    std::cout << "is_identity=" << is_identity(id, 3) << "\n";
    std::cout << "matrix_mul_trace=" << matrix_mul_trace(m, b, 3) << "\n";
    std::cout << "main_diag_product=" << main_diag_product(m, 3) << "\n";
    std::cout << "max_element=" << max_element(m, 3) << "\n";
    std::cout << "min_element=" << min_element(m, 3) << "\n";
    std::cout << "determinant_2x2=" << determinant_2x2(d2) << "\n";
    std::cout << "determinant_3x3=" << determinant_3x3(m) << "\n";
    return 0;
}
