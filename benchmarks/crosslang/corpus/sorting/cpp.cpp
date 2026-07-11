// Cross-language sorting suite (C++). Classic sort algorithms and order
// statistics over an i64 array and a length. Each function returns a scalar
// (a checksum sum(i*sorted[i]), a count, or an index). Functions that reorder
// copy into a local vector so the caller's data is left untouched.
#include <cstdint>
#include <iostream>
#include <vector>
#include <algorithm>

static std::int64_t checksum(const std::vector<std::int64_t> &a, std::int64_t n) {
    std::int64_t sum = 0;
    for (std::int64_t i = 0; i < n; i++) sum += i * a[i];
    return sum;
}

std::int64_t insertion_sort_checksum(const std::int64_t *a, std::int64_t n) {
    std::vector<std::int64_t> buf(a, a + n);
    for (std::int64_t i = 1; i < n; i++) {
        std::int64_t key = buf[i], j = i - 1;
        while (j >= 0 && buf[j] > key) { buf[j + 1] = buf[j]; j--; }
        buf[j + 1] = key;
    }
    return checksum(buf, n);
}

std::int64_t selection_sort_checksum(const std::int64_t *a, std::int64_t n) {
    std::vector<std::int64_t> buf(a, a + n);
    for (std::int64_t i = 0; i < n - 1; i++) {
        std::int64_t mi = i;
        for (std::int64_t j = i + 1; j < n; j++) if (buf[j] < buf[mi]) mi = j;
        std::swap(buf[i], buf[mi]);
    }
    return checksum(buf, n);
}

std::int64_t bubble_sort_swaps(const std::int64_t *a, std::int64_t n) {
    std::vector<std::int64_t> buf(a, a + n);
    std::int64_t swaps = 0;
    for (std::int64_t i = 0; i < n; i++)
        for (std::int64_t j = 0; j + 1 + i < n; j++)
            if (buf[j] > buf[j + 1]) { std::swap(buf[j], buf[j + 1]); swaps++; }
    return swaps;
}

std::int64_t gnome_sort_checksum(const std::int64_t *a, std::int64_t n) {
    std::vector<std::int64_t> buf(a, a + n);
    std::int64_t i = 0;
    while (i < n) {
        if (i == 0 || buf[i] >= buf[i - 1]) i++;
        else { std::swap(buf[i], buf[i - 1]); i--; }
    }
    return checksum(buf, n);
}

std::int64_t cocktail_sort_checksum(const std::int64_t *a, std::int64_t n) {
    std::vector<std::int64_t> buf(a, a + n);
    std::int64_t lo = 0, hi = n - 1;
    bool swapped = true;
    while (swapped) {
        swapped = false;
        for (std::int64_t i = lo; i < hi; i++)
            if (buf[i] > buf[i + 1]) { std::swap(buf[i], buf[i + 1]); swapped = true; }
        if (!swapped) break;
        hi--;
        swapped = false;
        for (std::int64_t i = hi - 1; i >= lo; i--)
            if (buf[i] > buf[i + 1]) { std::swap(buf[i], buf[i + 1]); swapped = true; }
        lo++;
    }
    return checksum(buf, n);
}

std::int64_t comb_sort_checksum(const std::int64_t *a, std::int64_t n) {
    std::vector<std::int64_t> buf(a, a + n);
    std::int64_t gap = n;
    bool swapped = true;
    while (gap > 1 || swapped) {
        gap = (gap * 10) / 13;
        if (gap < 1) gap = 1;
        swapped = false;
        for (std::int64_t i = 0; i + gap < n; i++)
            if (buf[i] > buf[i + gap]) { std::swap(buf[i], buf[i + gap]); swapped = true; }
    }
    return checksum(buf, n);
}

std::int64_t count_inversions(const std::int64_t *a, std::int64_t n) {
    std::int64_t count = 0;
    for (std::int64_t i = 0; i < n; i++)
        for (std::int64_t j = i + 1; j < n; j++) if (a[i] > a[j]) count++;
    return count;
}

std::int64_t is_sorted_desc(const std::int64_t *a, std::int64_t n) {
    for (std::int64_t i = 1; i < n; i++) if (a[i] > a[i - 1]) return 0;
    return 1;
}

std::int64_t merge_two_sorted_checksum(const std::int64_t *a, std::int64_t la, const std::int64_t *b, std::int64_t lb) {
    std::int64_t i = 0, j = 0, k = 0, sum = 0;
    while (i < la && j < lb) {
        if (a[i] <= b[j]) { sum += k * a[i]; i++; }
        else { sum += k * b[j]; j++; }
        k++;
    }
    while (i < la) { sum += k * a[i]; i++; k++; }
    while (j < lb) { sum += k * b[j]; j++; k++; }
    return sum;
}

std::int64_t partition_lomuto_index(const std::int64_t *a, std::int64_t n) {
    std::vector<std::int64_t> buf(a, a + n);
    std::int64_t pivot = buf[n - 1], i = 0;
    for (std::int64_t j = 0; j < n - 1; j++)
        if (buf[j] < pivot) { std::swap(buf[i], buf[j]); i++; }
    std::swap(buf[i], buf[n - 1]);
    return i;
}

std::int64_t kth_smallest(const std::int64_t *a, std::int64_t n, std::int64_t k) {
    std::vector<std::int64_t> buf(a, a + n);
    for (std::int64_t i = 0; i < k; i++) {
        std::int64_t mi = i;
        for (std::int64_t j = i + 1; j < n; j++) if (buf[j] < buf[mi]) mi = j;
        std::swap(buf[i], buf[mi]);
    }
    return buf[k - 1];
}

std::int64_t count_sorted_runs(const std::int64_t *a, std::int64_t n) {
    if (n == 0) return 0;
    std::int64_t runs = 1;
    for (std::int64_t i = 1; i < n; i++) if (a[i] < a[i - 1]) runs++;
    return runs;
}

std::int64_t min_swaps_selection(const std::int64_t *a, std::int64_t n) {
    std::vector<std::int64_t> buf(a, a + n);
    std::int64_t swaps = 0;
    for (std::int64_t i = 0; i < n - 1; i++) {
        std::int64_t mi = i;
        for (std::int64_t j = i + 1; j < n; j++) if (buf[j] < buf[mi]) mi = j;
        if (mi != i) { std::swap(buf[i], buf[mi]); swaps++; }
    }
    return swaps;
}

std::int64_t sorted_median(const std::int64_t *a, std::int64_t n) {
    std::vector<std::int64_t> buf(a, a + n);
    std::sort(buf.begin(), buf.end());
    return buf[n / 2];
}

std::int64_t sort_evens_first_checksum(const std::int64_t *a, std::int64_t n) {
    std::vector<std::int64_t> buf(a, a + n);
    std::sort(buf.begin(), buf.end());
    std::int64_t sum = 0, k = 0;
    for (std::int64_t i = 0; i < n; i++)
        if (buf[i] % 2 == 0) { sum += k * buf[i]; k++; }
    for (std::int64_t i = 0; i < n; i++)
        if (buf[i] % 2 != 0) { sum += k * buf[i]; k++; }
    return sum;
}

std::int64_t reverse_checksum(const std::int64_t *a, std::int64_t n) {
    std::int64_t sum = 0;
    for (std::int64_t i = 0; i < n; i++) sum += i * a[n - 1 - i];
    return sum;
}

std::int64_t max_gap_sorted(const std::int64_t *a, std::int64_t n) {
    std::vector<std::int64_t> buf(a, a + n);
    std::sort(buf.begin(), buf.end());
    std::int64_t g = 0;
    for (std::int64_t i = 1; i < n; i++) {
        std::int64_t d = buf[i] - buf[i - 1];
        if (d > g) g = d;
    }
    return g;
}

std::int64_t second_smallest(const std::int64_t *a, std::int64_t n) {
    std::int64_t first = a[0], second = a[1];
    if (second < first) std::swap(first, second);
    for (std::int64_t i = 2; i < n; i++) {
        if (a[i] < first) { second = first; first = a[i]; }
        else if (a[i] < second) second = a[i];
    }
    return second;
}

int main() {
    std::int64_t t[6] = { 5, 3, 8, 1, 9, 2 };
    std::int64_t p[4] = { 1, 4, 6, 8 };
    std::int64_t q[5] = { 2, 3, 5, 7, 9 };
    std::cout << "insertion_sort_checksum=" << insertion_sort_checksum(t, 6) << "\n";
    std::cout << "selection_sort_checksum=" << selection_sort_checksum(t, 6) << "\n";
    std::cout << "bubble_sort_swaps=" << bubble_sort_swaps(t, 6) << "\n";
    std::cout << "gnome_sort_checksum=" << gnome_sort_checksum(t, 6) << "\n";
    std::cout << "cocktail_sort_checksum=" << cocktail_sort_checksum(t, 6) << "\n";
    std::cout << "comb_sort_checksum=" << comb_sort_checksum(t, 6) << "\n";
    std::cout << "count_inversions=" << count_inversions(t, 6) << "\n";
    std::cout << "is_sorted_desc=" << is_sorted_desc(t, 6) << "\n";
    std::cout << "merge_two_sorted_checksum=" << merge_two_sorted_checksum(p, 4, q, 5) << "\n";
    std::cout << "partition_lomuto_index=" << partition_lomuto_index(t, 6) << "\n";
    std::cout << "kth_smallest=" << kth_smallest(t, 6, 3) << "\n";
    std::cout << "count_sorted_runs=" << count_sorted_runs(t, 6) << "\n";
    std::cout << "min_swaps_selection=" << min_swaps_selection(t, 6) << "\n";
    std::cout << "sorted_median=" << sorted_median(t, 6) << "\n";
    std::cout << "sort_evens_first_checksum=" << sort_evens_first_checksum(t, 6) << "\n";
    std::cout << "reverse_checksum=" << reverse_checksum(t, 6) << "\n";
    std::cout << "max_gap_sorted=" << max_gap_sorted(t, 6) << "\n";
    std::cout << "second_smallest=" << second_smallest(t, 6) << "\n";
    return 0;
}
