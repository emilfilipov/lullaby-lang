/* Cross-language arrays suite (C). Real-world array/statistics operations
   over an i64 array and a length. bubble_sort_checksum copies into a local
   buffer so the caller's array is left untouched. */
#include <stdio.h>
#include <stdint.h>

int64_t sum_array(const int64_t *a, int64_t n) {
    int64_t sum = 0;
    for (int64_t i = 0; i < n; i++) sum += a[i];
    return sum;
}

int64_t max_array(const int64_t *a, int64_t n) {
    int64_t m = a[0];
    for (int64_t i = 1; i < n; i++) if (a[i] > m) m = a[i];
    return m;
}

int64_t min_array(const int64_t *a, int64_t n) {
    int64_t m = a[0];
    for (int64_t i = 1; i < n; i++) if (a[i] < m) m = a[i];
    return m;
}

int64_t mean_floor(const int64_t *a, int64_t n) {
    return sum_array(a, n) / n;
}

int64_t count_positive(const int64_t *a, int64_t n) {
    int64_t count = 0;
    for (int64_t i = 0; i < n; i++) if (a[i] > 0) count++;
    return count;
}

int64_t count_equal(const int64_t *a, int64_t n, int64_t x) {
    int64_t count = 0;
    for (int64_t i = 0; i < n; i++) if (a[i] == x) count++;
    return count;
}

int64_t index_of(const int64_t *a, int64_t n, int64_t x) {
    for (int64_t i = 0; i < n; i++) if (a[i] == x) return i;
    return -1;
}

int64_t binary_search(const int64_t *a, int64_t n, int64_t x) {
    int64_t lo = 0, hi = n - 1;
    while (lo <= hi) {
        int64_t mid = (lo + hi) / 2;
        if (a[mid] == x) return mid;
        else if (a[mid] < x) lo = mid + 1;
        else hi = mid - 1;
    }
    return -1;
}

int64_t is_sorted_asc(const int64_t *a, int64_t n) {
    for (int64_t i = 1; i < n; i++) if (a[i] < a[i - 1]) return 0;
    return 1;
}

int64_t range_span(const int64_t *a, int64_t n) {
    return max_array(a, n) - min_array(a, n);
}

int64_t dot_product(const int64_t *a, const int64_t *b, int64_t n) {
    int64_t sum = 0;
    for (int64_t i = 0; i < n; i++) sum += a[i] * b[i];
    return sum;
}

int64_t count_distinct_sorted(const int64_t *a, int64_t n) {
    if (n == 0) return 0;
    int64_t count = 1;
    for (int64_t i = 1; i < n; i++) if (a[i] != a[i - 1]) count++;
    return count;
}

int64_t second_largest(const int64_t *a, int64_t n) {
    int64_t first = a[0], second = a[1];
    if (second > first) { int64_t t = first; first = second; second = t; }
    for (int64_t i = 2; i < n; i++) {
        if (a[i] > first) { second = first; first = a[i]; }
        else if (a[i] > second) second = a[i];
    }
    return second;
}

int64_t prefix_sum_last(const int64_t *a, int64_t n) {
    int64_t prefix = 0;
    for (int64_t i = 0; i < n; i++) prefix += a[i];
    return prefix;
}

int64_t bubble_sort_checksum(const int64_t *a, int64_t n) {
    int64_t buf[256];
    for (int64_t i = 0; i < n; i++) buf[i] = a[i];
    for (int64_t i = 0; i < n; i++)
        for (int64_t j = 0; j + 1 + i < n; j++)
            if (buf[j] > buf[j + 1]) {
                int64_t t = buf[j]; buf[j] = buf[j + 1]; buf[j + 1] = t;
            }
    int64_t sum = 0;
    for (int64_t i = 0; i < n; i++) sum += i * buf[i];
    return sum;
}

int main(void) {
    int64_t t[6] = { 5, 3, 8, 1, 9, 2 };
    int64_t s[6] = { 1, 2, 2, 3, 5, 8 };
    printf("sum_array=%lld\n", (long long)sum_array(t, 6));
    printf("max_array=%lld\n", (long long)max_array(t, 6));
    printf("min_array=%lld\n", (long long)min_array(t, 6));
    printf("mean_floor=%lld\n", (long long)mean_floor(t, 6));
    printf("count_positive=%lld\n", (long long)count_positive(t, 6));
    printf("count_equal=%lld\n", (long long)count_equal(t, 6, 8));
    printf("index_of=%lld\n", (long long)index_of(t, 6, 1));
    printf("binary_search=%lld\n", (long long)binary_search(s, 6, 5));
    printf("is_sorted_asc=%lld\n", (long long)is_sorted_asc(s, 6));
    printf("range_span=%lld\n", (long long)range_span(t, 6));
    printf("dot_product=%lld\n", (long long)dot_product(t, s, 6));
    printf("count_distinct_sorted=%lld\n", (long long)count_distinct_sorted(s, 6));
    printf("second_largest=%lld\n", (long long)second_largest(t, 6));
    printf("prefix_sum_last=%lld\n", (long long)prefix_sum_last(t, 6));
    printf("bubble_sort_checksum=%lld\n", (long long)bubble_sort_checksum(t, 6));
    return 0;
}
