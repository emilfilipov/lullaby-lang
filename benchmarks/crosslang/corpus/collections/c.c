/* Cross-language collections suite (C). Array-as-collection algorithms:
   frequency, grouping, and set-like operations over an i64 array and a length.
   No hash maps are used: everything is counting and scanning, relying on
   sorted inputs where noted. */
#include <stdio.h>
#include <stdint.h>

int64_t count_frequency_of(const int64_t *a, int64_t n, int64_t x) {
    int64_t count = 0;
    for (int64_t i = 0; i < n; i++) if (a[i] == x) count++;
    return count;
}

int64_t max_frequency(const int64_t *a, int64_t n) {
    if (n == 0) return 0;
    int64_t best = 1, run = 1;
    for (int64_t i = 1; i < n; i++) {
        run = (a[i] == a[i - 1]) ? run + 1 : 1;
        if (run > best) best = run;
    }
    return best;
}

int64_t first_duplicate_value(const int64_t *a, int64_t n) {
    for (int64_t i = 1; i < n; i++) if (a[i] == a[i - 1]) return a[i];
    return -1;
}

int64_t has_pair_sum(const int64_t *a, int64_t n, int64_t target) {
    int64_t lo = 0, hi = n - 1;
    while (lo < hi) {
        int64_t s = a[lo] + a[hi];
        if (s == target) return 1;
        else if (s < target) lo++;
        else hi--;
    }
    return 0;
}

int64_t count_distinct_sorted(const int64_t *a, int64_t n) {
    if (n == 0) return 0;
    int64_t count = 1;
    for (int64_t i = 1; i < n; i++) if (a[i] != a[i - 1]) count++;
    return count;
}

int64_t most_common_sorted(const int64_t *a, int64_t n) {
    int64_t best_val = a[0], best = 1, run = 1;
    for (int64_t i = 1; i < n; i++) {
        run = (a[i] == a[i - 1]) ? run + 1 : 1;
        if (run > best) { best = run; best_val = a[i]; }
    }
    return best_val;
}

int64_t count_even(const int64_t *a, int64_t n) {
    int64_t count = 0;
    for (int64_t i = 0; i < n; i++) if (a[i] % 2 == 0) count++;
    return count;
}

int64_t count_odd(const int64_t *a, int64_t n) {
    int64_t count = 0;
    for (int64_t i = 0; i < n; i++) if (a[i] % 2 != 0) count++;
    return count;
}

int64_t partition_point(const int64_t *a, int64_t n) {
    for (int64_t i = 0; i < n; i++) if (a[i] >= 0) return i;
    return n;
}

int64_t count_in_range(const int64_t *a, int64_t n, int64_t lo, int64_t hi) {
    int64_t count = 0;
    for (int64_t i = 0; i < n; i++) if (a[i] >= lo && a[i] <= hi) count++;
    return count;
}

int64_t running_total_last(const int64_t *a, int64_t n) {
    int64_t total = 0;
    for (int64_t i = 0; i < n; i++) total += a[i];
    return total;
}

int64_t zip_sum(const int64_t *a, const int64_t *b, int64_t n) {
    int64_t sum = 0;
    for (int64_t i = 0; i < n; i++) sum += a[i] + b[i];
    return sum;
}

int64_t intersect_count_sorted(const int64_t *a, int64_t la, const int64_t *b, int64_t lb) {
    int64_t i = 0, j = 0, count = 0;
    while (i < la && j < lb) {
        if (a[i] == b[j]) { count++; i++; j++; }
        else if (a[i] < b[j]) i++;
        else j++;
    }
    return count;
}

int64_t union_count_sorted(const int64_t *a, int64_t la, const int64_t *b, int64_t lb) {
    int64_t i = 0, j = 0, count = 0;
    while (i < la && j < lb) {
        if (a[i] == b[j]) { count++; i++; j++; }
        else if (a[i] < b[j]) { count++; i++; }
        else { count++; j++; }
    }
    while (i < la) { count++; i++; }
    while (j < lb) { count++; j++; }
    return count;
}

int64_t is_subset_sorted(const int64_t *a, int64_t la, const int64_t *b, int64_t lb) {
    int64_t i = 0, j = 0;
    while (i < la && j < lb) {
        if (a[i] == b[j]) { i++; j++; }
        else if (a[i] > b[j]) j++;
        else return 0;
    }
    return i < la ? 0 : 1;
}

int64_t rotate_left_checksum(const int64_t *a, int64_t n, int64_t k) {
    int64_t shift = k % n, sum = 0;
    for (int64_t i = 0; i < n; i++) {
        int64_t idx = i + shift;
        if (idx >= n) idx -= n;
        sum += i * a[idx];
    }
    return sum;
}

int64_t dedup_sorted_checksum(const int64_t *a, int64_t n) {
    if (n == 0) return 0;
    int64_t sum = 0, pos = 0, prev = a[0];
    for (int64_t i = 1; i < n; i++) {
        if (a[i] != prev) { pos++; sum += pos * a[i]; prev = a[i]; }
    }
    return sum;
}

int64_t chunk_sum_max(const int64_t *a, int64_t n, int64_t k) {
    int64_t window = 0;
    for (int64_t i = 0; i < k; i++) window += a[i];
    int64_t best = window;
    for (int64_t i = k; i < n; i++) {
        window += a[i] - a[i - k];
        if (window > best) best = window;
    }
    return best;
}

int main(void) {
    int64_t a[10] = { -5, -2, -2, 0, 1, 1, 1, 4, 7, 7 };
    int64_t b[10] = { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 };
    int64_t c[5] = { 1, 3, 5, 7, 9 };
    int64_t d[5] = { 2, 3, 5, 8, 9 };
    int64_t e[3] = { 3, 5, 9 };
    printf("count_frequency_of=%lld\n", (long long)count_frequency_of(a, 10, 1));
    printf("max_frequency=%lld\n", (long long)max_frequency(a, 10));
    printf("first_duplicate_value=%lld\n", (long long)first_duplicate_value(a, 10));
    printf("has_pair_sum=%lld\n", (long long)has_pair_sum(a, 10, 2));
    printf("count_distinct_sorted=%lld\n", (long long)count_distinct_sorted(a, 10));
    printf("most_common_sorted=%lld\n", (long long)most_common_sorted(a, 10));
    printf("count_even=%lld\n", (long long)count_even(a, 10));
    printf("count_odd=%lld\n", (long long)count_odd(a, 10));
    printf("partition_point=%lld\n", (long long)partition_point(a, 10));
    printf("count_in_range=%lld\n", (long long)count_in_range(a, 10, -2, 1));
    printf("running_total_last=%lld\n", (long long)running_total_last(a, 10));
    printf("zip_sum=%lld\n", (long long)zip_sum(a, b, 10));
    printf("intersect_count_sorted=%lld\n", (long long)intersect_count_sorted(c, 5, d, 5));
    printf("union_count_sorted=%lld\n", (long long)union_count_sorted(c, 5, d, 5));
    printf("is_subset_sorted=%lld\n", (long long)is_subset_sorted(e, 3, d, 5));
    printf("rotate_left_checksum=%lld\n", (long long)rotate_left_checksum(b, 10, 3));
    printf("dedup_sorted_checksum=%lld\n", (long long)dedup_sorted_checksum(a, 10));
    printf("chunk_sum_max=%lld\n", (long long)chunk_sum_max(b, 10, 3));
    return 0;
}
