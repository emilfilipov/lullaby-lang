// Cross-language collections suite (C++). Array-as-collection algorithms:
// frequency, grouping, and set-like operations over an i64 array and a length.
// No hash maps are used: everything is counting and scanning, relying on
// sorted inputs where noted.
#include <cstdint>
#include <iostream>

std::int64_t count_frequency_of(const std::int64_t *a, std::int64_t n, std::int64_t x) {
    std::int64_t count = 0;
    for (std::int64_t i = 0; i < n; i++) if (a[i] == x) count++;
    return count;
}

std::int64_t max_frequency(const std::int64_t *a, std::int64_t n) {
    if (n == 0) return 0;
    std::int64_t best = 1, run = 1;
    for (std::int64_t i = 1; i < n; i++) {
        run = (a[i] == a[i - 1]) ? run + 1 : 1;
        if (run > best) best = run;
    }
    return best;
}

std::int64_t first_duplicate_value(const std::int64_t *a, std::int64_t n) {
    for (std::int64_t i = 1; i < n; i++) if (a[i] == a[i - 1]) return a[i];
    return -1;
}

std::int64_t has_pair_sum(const std::int64_t *a, std::int64_t n, std::int64_t target) {
    std::int64_t lo = 0, hi = n - 1;
    while (lo < hi) {
        std::int64_t s = a[lo] + a[hi];
        if (s == target) return 1;
        else if (s < target) lo++;
        else hi--;
    }
    return 0;
}

std::int64_t count_distinct_sorted(const std::int64_t *a, std::int64_t n) {
    if (n == 0) return 0;
    std::int64_t count = 1;
    for (std::int64_t i = 1; i < n; i++) if (a[i] != a[i - 1]) count++;
    return count;
}

std::int64_t most_common_sorted(const std::int64_t *a, std::int64_t n) {
    std::int64_t best_val = a[0], best = 1, run = 1;
    for (std::int64_t i = 1; i < n; i++) {
        run = (a[i] == a[i - 1]) ? run + 1 : 1;
        if (run > best) { best = run; best_val = a[i]; }
    }
    return best_val;
}

std::int64_t count_even(const std::int64_t *a, std::int64_t n) {
    std::int64_t count = 0;
    for (std::int64_t i = 0; i < n; i++) if (a[i] % 2 == 0) count++;
    return count;
}

std::int64_t count_odd(const std::int64_t *a, std::int64_t n) {
    std::int64_t count = 0;
    for (std::int64_t i = 0; i < n; i++) if (a[i] % 2 != 0) count++;
    return count;
}

std::int64_t partition_point(const std::int64_t *a, std::int64_t n) {
    for (std::int64_t i = 0; i < n; i++) if (a[i] >= 0) return i;
    return n;
}

std::int64_t count_in_range(const std::int64_t *a, std::int64_t n, std::int64_t lo, std::int64_t hi) {
    std::int64_t count = 0;
    for (std::int64_t i = 0; i < n; i++) if (a[i] >= lo && a[i] <= hi) count++;
    return count;
}

std::int64_t running_total_last(const std::int64_t *a, std::int64_t n) {
    std::int64_t total = 0;
    for (std::int64_t i = 0; i < n; i++) total += a[i];
    return total;
}

std::int64_t zip_sum(const std::int64_t *a, const std::int64_t *b, std::int64_t n) {
    std::int64_t sum = 0;
    for (std::int64_t i = 0; i < n; i++) sum += a[i] + b[i];
    return sum;
}

std::int64_t intersect_count_sorted(const std::int64_t *a, std::int64_t la, const std::int64_t *b, std::int64_t lb) {
    std::int64_t i = 0, j = 0, count = 0;
    while (i < la && j < lb) {
        if (a[i] == b[j]) { count++; i++; j++; }
        else if (a[i] < b[j]) i++;
        else j++;
    }
    return count;
}

std::int64_t union_count_sorted(const std::int64_t *a, std::int64_t la, const std::int64_t *b, std::int64_t lb) {
    std::int64_t i = 0, j = 0, count = 0;
    while (i < la && j < lb) {
        if (a[i] == b[j]) { count++; i++; j++; }
        else if (a[i] < b[j]) { count++; i++; }
        else { count++; j++; }
    }
    while (i < la) { count++; i++; }
    while (j < lb) { count++; j++; }
    return count;
}

std::int64_t is_subset_sorted(const std::int64_t *a, std::int64_t la, const std::int64_t *b, std::int64_t lb) {
    std::int64_t i = 0, j = 0;
    while (i < la && j < lb) {
        if (a[i] == b[j]) { i++; j++; }
        else if (a[i] > b[j]) j++;
        else return 0;
    }
    return i < la ? 0 : 1;
}

std::int64_t rotate_left_checksum(const std::int64_t *a, std::int64_t n, std::int64_t k) {
    std::int64_t shift = k % n, sum = 0;
    for (std::int64_t i = 0; i < n; i++) {
        std::int64_t idx = i + shift;
        if (idx >= n) idx -= n;
        sum += i * a[idx];
    }
    return sum;
}

std::int64_t dedup_sorted_checksum(const std::int64_t *a, std::int64_t n) {
    if (n == 0) return 0;
    std::int64_t sum = 0, pos = 0, prev = a[0];
    for (std::int64_t i = 1; i < n; i++) {
        if (a[i] != prev) { pos++; sum += pos * a[i]; prev = a[i]; }
    }
    return sum;
}

std::int64_t chunk_sum_max(const std::int64_t *a, std::int64_t n, std::int64_t k) {
    std::int64_t window = 0;
    for (std::int64_t i = 0; i < k; i++) window += a[i];
    std::int64_t best = window;
    for (std::int64_t i = k; i < n; i++) {
        window += a[i] - a[i - k];
        if (window > best) best = window;
    }
    return best;
}

int main() {
    std::int64_t a[10] = { -5, -2, -2, 0, 1, 1, 1, 4, 7, 7 };
    std::int64_t b[10] = { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 };
    std::int64_t c[5] = { 1, 3, 5, 7, 9 };
    std::int64_t d[5] = { 2, 3, 5, 8, 9 };
    std::int64_t e[3] = { 3, 5, 9 };
    std::cout << "count_frequency_of=" << count_frequency_of(a, 10, 1) << "\n";
    std::cout << "max_frequency=" << max_frequency(a, 10) << "\n";
    std::cout << "first_duplicate_value=" << first_duplicate_value(a, 10) << "\n";
    std::cout << "has_pair_sum=" << has_pair_sum(a, 10, 2) << "\n";
    std::cout << "count_distinct_sorted=" << count_distinct_sorted(a, 10) << "\n";
    std::cout << "most_common_sorted=" << most_common_sorted(a, 10) << "\n";
    std::cout << "count_even=" << count_even(a, 10) << "\n";
    std::cout << "count_odd=" << count_odd(a, 10) << "\n";
    std::cout << "partition_point=" << partition_point(a, 10) << "\n";
    std::cout << "count_in_range=" << count_in_range(a, 10, -2, 1) << "\n";
    std::cout << "running_total_last=" << running_total_last(a, 10) << "\n";
    std::cout << "zip_sum=" << zip_sum(a, b, 10) << "\n";
    std::cout << "intersect_count_sorted=" << intersect_count_sorted(c, 5, d, 5) << "\n";
    std::cout << "union_count_sorted=" << union_count_sorted(c, 5, d, 5) << "\n";
    std::cout << "is_subset_sorted=" << is_subset_sorted(e, 3, d, 5) << "\n";
    std::cout << "rotate_left_checksum=" << rotate_left_checksum(b, 10, 3) << "\n";
    std::cout << "dedup_sorted_checksum=" << dedup_sorted_checksum(a, 10) << "\n";
    std::cout << "chunk_sum_max=" << chunk_sum_max(b, 10, 3) << "\n";
    return 0;
}
