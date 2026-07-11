// Cross-language arrays suite (C++). Real-world array/statistics operations
// over an i64 array and a length. bubble_sort_checksum copies into a local
// vector so the caller's data is left untouched.
#include <cstdint>
#include <iostream>
#include <vector>

std::int64_t sum_array(const std::int64_t *a, std::int64_t n) {
    std::int64_t sum = 0;
    for (std::int64_t i = 0; i < n; i++) sum += a[i];
    return sum;
}

std::int64_t max_array(const std::int64_t *a, std::int64_t n) {
    std::int64_t m = a[0];
    for (std::int64_t i = 1; i < n; i++) if (a[i] > m) m = a[i];
    return m;
}

std::int64_t min_array(const std::int64_t *a, std::int64_t n) {
    std::int64_t m = a[0];
    for (std::int64_t i = 1; i < n; i++) if (a[i] < m) m = a[i];
    return m;
}

std::int64_t mean_floor(const std::int64_t *a, std::int64_t n) {
    return sum_array(a, n) / n;
}

std::int64_t count_positive(const std::int64_t *a, std::int64_t n) {
    std::int64_t count = 0;
    for (std::int64_t i = 0; i < n; i++) if (a[i] > 0) count++;
    return count;
}

std::int64_t count_equal(const std::int64_t *a, std::int64_t n, std::int64_t x) {
    std::int64_t count = 0;
    for (std::int64_t i = 0; i < n; i++) if (a[i] == x) count++;
    return count;
}

std::int64_t index_of(const std::int64_t *a, std::int64_t n, std::int64_t x) {
    for (std::int64_t i = 0; i < n; i++) if (a[i] == x) return i;
    return -1;
}

std::int64_t binary_search(const std::int64_t *a, std::int64_t n, std::int64_t x) {
    std::int64_t lo = 0, hi = n - 1;
    while (lo <= hi) {
        std::int64_t mid = (lo + hi) / 2;
        if (a[mid] == x) return mid;
        else if (a[mid] < x) lo = mid + 1;
        else hi = mid - 1;
    }
    return -1;
}

std::int64_t is_sorted_asc(const std::int64_t *a, std::int64_t n) {
    for (std::int64_t i = 1; i < n; i++) if (a[i] < a[i - 1]) return 0;
    return 1;
}

std::int64_t range_span(const std::int64_t *a, std::int64_t n) {
    return max_array(a, n) - min_array(a, n);
}

std::int64_t dot_product(const std::int64_t *a, const std::int64_t *b, std::int64_t n) {
    std::int64_t sum = 0;
    for (std::int64_t i = 0; i < n; i++) sum += a[i] * b[i];
    return sum;
}

std::int64_t count_distinct_sorted(const std::int64_t *a, std::int64_t n) {
    if (n == 0) return 0;
    std::int64_t count = 1;
    for (std::int64_t i = 1; i < n; i++) if (a[i] != a[i - 1]) count++;
    return count;
}

std::int64_t second_largest(const std::int64_t *a, std::int64_t n) {
    std::int64_t first = a[0], second = a[1];
    if (second > first) std::swap(first, second);
    for (std::int64_t i = 2; i < n; i++) {
        if (a[i] > first) { second = first; first = a[i]; }
        else if (a[i] > second) second = a[i];
    }
    return second;
}

std::int64_t prefix_sum_last(const std::int64_t *a, std::int64_t n) {
    std::int64_t prefix = 0;
    for (std::int64_t i = 0; i < n; i++) prefix += a[i];
    return prefix;
}

std::int64_t bubble_sort_checksum(const std::int64_t *a, std::int64_t n) {
    std::vector<std::int64_t> buf(a, a + n);
    for (std::int64_t i = 0; i < n; i++)
        for (std::int64_t j = 0; j + 1 + i < n; j++)
            if (buf[j] > buf[j + 1]) std::swap(buf[j], buf[j + 1]);
    std::int64_t sum = 0;
    for (std::int64_t i = 0; i < n; i++) sum += i * buf[i];
    return sum;
}

int main() {
    std::int64_t t[6] = { 5, 3, 8, 1, 9, 2 };
    std::int64_t s[6] = { 1, 2, 2, 3, 5, 8 };
    std::cout << "sum_array=" << sum_array(t, 6) << "\n";
    std::cout << "max_array=" << max_array(t, 6) << "\n";
    std::cout << "min_array=" << min_array(t, 6) << "\n";
    std::cout << "mean_floor=" << mean_floor(t, 6) << "\n";
    std::cout << "count_positive=" << count_positive(t, 6) << "\n";
    std::cout << "count_equal=" << count_equal(t, 6, 8) << "\n";
    std::cout << "index_of=" << index_of(t, 6, 1) << "\n";
    std::cout << "binary_search=" << binary_search(s, 6, 5) << "\n";
    std::cout << "is_sorted_asc=" << is_sorted_asc(s, 6) << "\n";
    std::cout << "range_span=" << range_span(t, 6) << "\n";
    std::cout << "dot_product=" << dot_product(t, s, 6) << "\n";
    std::cout << "count_distinct_sorted=" << count_distinct_sorted(s, 6) << "\n";
    std::cout << "second_largest=" << second_largest(t, 6) << "\n";
    std::cout << "prefix_sum_last=" << prefix_sum_last(t, 6) << "\n";
    std::cout << "bubble_sort_checksum=" << bubble_sort_checksum(t, 6) << "\n";
    return 0;
}
