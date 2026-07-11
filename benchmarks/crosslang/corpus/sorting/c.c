/* Cross-language sorting suite (C). Classic sort algorithms and order
   statistics over an i64 array and a length. Each function returns a scalar
   (a checksum sum(i*sorted[i]), a count, or an index). Functions that reorder
   copy into a local buffer so the caller's array is left untouched. */
#include <stdio.h>
#include <stdint.h>

static int64_t checksum(const int64_t *a, int64_t n) {
    int64_t sum = 0;
    for (int64_t i = 0; i < n; i++) sum += i * a[i];
    return sum;
}

int64_t insertion_sort_checksum(const int64_t *a, int64_t n) {
    int64_t buf[256];
    for (int64_t i = 0; i < n; i++) buf[i] = a[i];
    for (int64_t i = 1; i < n; i++) {
        int64_t key = buf[i], j = i - 1;
        while (j >= 0 && buf[j] > key) { buf[j + 1] = buf[j]; j--; }
        buf[j + 1] = key;
    }
    return checksum(buf, n);
}

int64_t selection_sort_checksum(const int64_t *a, int64_t n) {
    int64_t buf[256];
    for (int64_t i = 0; i < n; i++) buf[i] = a[i];
    for (int64_t i = 0; i < n - 1; i++) {
        int64_t mi = i;
        for (int64_t j = i + 1; j < n; j++) if (buf[j] < buf[mi]) mi = j;
        int64_t t = buf[i]; buf[i] = buf[mi]; buf[mi] = t;
    }
    return checksum(buf, n);
}

int64_t bubble_sort_swaps(const int64_t *a, int64_t n) {
    int64_t buf[256];
    for (int64_t i = 0; i < n; i++) buf[i] = a[i];
    int64_t swaps = 0;
    for (int64_t i = 0; i < n; i++)
        for (int64_t j = 0; j + 1 + i < n; j++)
            if (buf[j] > buf[j + 1]) {
                int64_t t = buf[j]; buf[j] = buf[j + 1]; buf[j + 1] = t;
                swaps++;
            }
    return swaps;
}

int64_t gnome_sort_checksum(const int64_t *a, int64_t n) {
    int64_t buf[256];
    for (int64_t i = 0; i < n; i++) buf[i] = a[i];
    int64_t i = 0;
    while (i < n) {
        if (i == 0 || buf[i] >= buf[i - 1]) i++;
        else {
            int64_t t = buf[i]; buf[i] = buf[i - 1]; buf[i - 1] = t;
            i--;
        }
    }
    return checksum(buf, n);
}

int64_t cocktail_sort_checksum(const int64_t *a, int64_t n) {
    int64_t buf[256];
    for (int64_t i = 0; i < n; i++) buf[i] = a[i];
    int64_t lo = 0, hi = n - 1, swapped = 1;
    while (swapped) {
        swapped = 0;
        for (int64_t i = lo; i < hi; i++)
            if (buf[i] > buf[i + 1]) {
                int64_t t = buf[i]; buf[i] = buf[i + 1]; buf[i + 1] = t;
                swapped = 1;
            }
        if (!swapped) break;
        hi--;
        swapped = 0;
        for (int64_t i = hi - 1; i >= lo; i--)
            if (buf[i] > buf[i + 1]) {
                int64_t t = buf[i]; buf[i] = buf[i + 1]; buf[i + 1] = t;
                swapped = 1;
            }
        lo++;
    }
    return checksum(buf, n);
}

int64_t comb_sort_checksum(const int64_t *a, int64_t n) {
    int64_t buf[256];
    for (int64_t i = 0; i < n; i++) buf[i] = a[i];
    int64_t gap = n, swapped = 1;
    while (gap > 1 || swapped) {
        gap = (gap * 10) / 13;
        if (gap < 1) gap = 1;
        swapped = 0;
        for (int64_t i = 0; i + gap < n; i++)
            if (buf[i] > buf[i + gap]) {
                int64_t t = buf[i]; buf[i] = buf[i + gap]; buf[i + gap] = t;
                swapped = 1;
            }
    }
    return checksum(buf, n);
}

int64_t count_inversions(const int64_t *a, int64_t n) {
    int64_t count = 0;
    for (int64_t i = 0; i < n; i++)
        for (int64_t j = i + 1; j < n; j++) if (a[i] > a[j]) count++;
    return count;
}

int64_t is_sorted_desc(const int64_t *a, int64_t n) {
    for (int64_t i = 1; i < n; i++) if (a[i] > a[i - 1]) return 0;
    return 1;
}

int64_t merge_two_sorted_checksum(const int64_t *a, int64_t la, const int64_t *b, int64_t lb) {
    int64_t i = 0, j = 0, k = 0, sum = 0;
    while (i < la && j < lb) {
        if (a[i] <= b[j]) { sum += k * a[i]; i++; }
        else { sum += k * b[j]; j++; }
        k++;
    }
    while (i < la) { sum += k * a[i]; i++; k++; }
    while (j < lb) { sum += k * b[j]; j++; k++; }
    return sum;
}

int64_t partition_lomuto_index(const int64_t *a, int64_t n) {
    int64_t buf[256];
    for (int64_t i = 0; i < n; i++) buf[i] = a[i];
    int64_t pivot = buf[n - 1], i = 0;
    for (int64_t j = 0; j < n - 1; j++)
        if (buf[j] < pivot) {
            int64_t t = buf[i]; buf[i] = buf[j]; buf[j] = t;
            i++;
        }
    int64_t t = buf[i]; buf[i] = buf[n - 1]; buf[n - 1] = t;
    return i;
}

int64_t kth_smallest(const int64_t *a, int64_t n, int64_t k) {
    int64_t buf[256];
    for (int64_t i = 0; i < n; i++) buf[i] = a[i];
    for (int64_t i = 0; i < k; i++) {
        int64_t mi = i;
        for (int64_t j = i + 1; j < n; j++) if (buf[j] < buf[mi]) mi = j;
        int64_t t = buf[i]; buf[i] = buf[mi]; buf[mi] = t;
    }
    return buf[k - 1];
}

int64_t count_sorted_runs(const int64_t *a, int64_t n) {
    if (n == 0) return 0;
    int64_t runs = 1;
    for (int64_t i = 1; i < n; i++) if (a[i] < a[i - 1]) runs++;
    return runs;
}

int64_t min_swaps_selection(const int64_t *a, int64_t n) {
    int64_t buf[256];
    for (int64_t i = 0; i < n; i++) buf[i] = a[i];
    int64_t swaps = 0;
    for (int64_t i = 0; i < n - 1; i++) {
        int64_t mi = i;
        for (int64_t j = i + 1; j < n; j++) if (buf[j] < buf[mi]) mi = j;
        if (mi != i) {
            int64_t t = buf[i]; buf[i] = buf[mi]; buf[mi] = t;
            swaps++;
        }
    }
    return swaps;
}

static void sort_asc(int64_t *buf, int64_t n) {
    for (int64_t i = 1; i < n; i++) {
        int64_t key = buf[i], j = i - 1;
        while (j >= 0 && buf[j] > key) { buf[j + 1] = buf[j]; j--; }
        buf[j + 1] = key;
    }
}

int64_t sorted_median(const int64_t *a, int64_t n) {
    int64_t buf[256];
    for (int64_t i = 0; i < n; i++) buf[i] = a[i];
    sort_asc(buf, n);
    return buf[n / 2];
}

int64_t sort_evens_first_checksum(const int64_t *a, int64_t n) {
    int64_t buf[256];
    for (int64_t i = 0; i < n; i++) buf[i] = a[i];
    sort_asc(buf, n);
    int64_t sum = 0, k = 0;
    for (int64_t i = 0; i < n; i++)
        if (buf[i] % 2 == 0) { sum += k * buf[i]; k++; }
    for (int64_t i = 0; i < n; i++)
        if (buf[i] % 2 != 0) { sum += k * buf[i]; k++; }
    return sum;
}

int64_t reverse_checksum(const int64_t *a, int64_t n) {
    int64_t sum = 0;
    for (int64_t i = 0; i < n; i++) sum += i * a[n - 1 - i];
    return sum;
}

int64_t max_gap_sorted(const int64_t *a, int64_t n) {
    int64_t buf[256];
    for (int64_t i = 0; i < n; i++) buf[i] = a[i];
    sort_asc(buf, n);
    int64_t g = 0;
    for (int64_t i = 1; i < n; i++) {
        int64_t d = buf[i] - buf[i - 1];
        if (d > g) g = d;
    }
    return g;
}

int64_t second_smallest(const int64_t *a, int64_t n) {
    int64_t first = a[0], second = a[1];
    if (second < first) { int64_t t = first; first = second; second = t; }
    for (int64_t i = 2; i < n; i++) {
        if (a[i] < first) { second = first; first = a[i]; }
        else if (a[i] < second) second = a[i];
    }
    return second;
}

int main(void) {
    int64_t t[6] = { 5, 3, 8, 1, 9, 2 };
    int64_t p[4] = { 1, 4, 6, 8 };
    int64_t q[5] = { 2, 3, 5, 7, 9 };
    printf("insertion_sort_checksum=%lld\n", (long long)insertion_sort_checksum(t, 6));
    printf("selection_sort_checksum=%lld\n", (long long)selection_sort_checksum(t, 6));
    printf("bubble_sort_swaps=%lld\n", (long long)bubble_sort_swaps(t, 6));
    printf("gnome_sort_checksum=%lld\n", (long long)gnome_sort_checksum(t, 6));
    printf("cocktail_sort_checksum=%lld\n", (long long)cocktail_sort_checksum(t, 6));
    printf("comb_sort_checksum=%lld\n", (long long)comb_sort_checksum(t, 6));
    printf("count_inversions=%lld\n", (long long)count_inversions(t, 6));
    printf("is_sorted_desc=%lld\n", (long long)is_sorted_desc(t, 6));
    printf("merge_two_sorted_checksum=%lld\n", (long long)merge_two_sorted_checksum(p, 4, q, 5));
    printf("partition_lomuto_index=%lld\n", (long long)partition_lomuto_index(t, 6));
    printf("kth_smallest=%lld\n", (long long)kth_smallest(t, 6, 3));
    printf("count_sorted_runs=%lld\n", (long long)count_sorted_runs(t, 6));
    printf("min_swaps_selection=%lld\n", (long long)min_swaps_selection(t, 6));
    printf("sorted_median=%lld\n", (long long)sorted_median(t, 6));
    printf("sort_evens_first_checksum=%lld\n", (long long)sort_evens_first_checksum(t, 6));
    printf("reverse_checksum=%lld\n", (long long)reverse_checksum(t, 6));
    printf("max_gap_sorted=%lld\n", (long long)max_gap_sorted(t, 6));
    printf("second_smallest=%lld\n", (long long)second_smallest(t, 6));
    return 0;
}
