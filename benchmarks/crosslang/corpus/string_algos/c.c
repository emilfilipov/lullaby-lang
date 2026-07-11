/* Cross-language string-algorithms suite (C). Classic string algorithms
   expressed over i64 ARRAYS of character codes, NOT string types, so all six
   languages run the identical array algorithm. edit_distance and lcs_length use
   a single rolling DP row. */
#include <stdio.h>
#include <stdint.h>

int64_t edit_distance(const int64_t *a, int64_t la, const int64_t *b, int64_t lb) {
    int64_t dp[64];
    for (int64_t j = 0; j <= lb; j++) dp[j] = j;
    for (int64_t i = 1; i <= la; i++) {
        int64_t prev = dp[0];
        dp[0] = i;
        for (int64_t j = 1; j <= lb; j++) {
            int64_t tmp = dp[j];
            if (a[i - 1] == b[j - 1]) {
                dp[j] = prev;
            } else {
                int64_t m = dp[j - 1];
                if (dp[j] < m) m = dp[j];
                if (prev < m) m = prev;
                dp[j] = m + 1;
            }
            prev = tmp;
        }
    }
    return dp[lb];
}

int64_t lcs_length(const int64_t *a, int64_t la, const int64_t *b, int64_t lb) {
    int64_t dp[64];
    for (int64_t j = 0; j <= lb; j++) dp[j] = 0;
    for (int64_t i = 1; i <= la; i++) {
        int64_t prev = 0;
        for (int64_t j = 1; j <= lb; j++) {
            int64_t tmp = dp[j];
            if (a[i - 1] == b[j - 1]) {
                dp[j] = prev + 1;
            } else if (dp[j - 1] > dp[j]) {
                dp[j] = dp[j - 1];
            }
            prev = tmp;
        }
    }
    return dp[lb];
}

int64_t hamming_distance(const int64_t *a, const int64_t *b, int64_t n) {
    int64_t d = 0;
    for (int64_t i = 0; i < n; i++) if (a[i] != b[i]) d++;
    return d;
}

int64_t longest_common_prefix_len(const int64_t *a, int64_t la, const int64_t *b, int64_t lb) {
    int64_t m = la < lb ? la : lb;
    int64_t i = 0;
    while (i < m) {
        if (a[i] != b[i]) return i;
        i++;
    }
    return i;
}

int64_t count_occurrences(const int64_t *text, int64_t tn, const int64_t *pat, int64_t pn) {
    if (pn == 0) return 0;
    int64_t count = 0;
    for (int64_t i = 0; i <= tn - pn; i++) {
        int64_t ok = 1;
        for (int64_t j = 0; j < pn; j++) {
            if (text[i + j] != pat[j]) { ok = 0; break; }
        }
        if (ok) count++;
    }
    return count;
}

int64_t is_rotation(const int64_t *a, const int64_t *b, int64_t n) {
    if (n == 0) return 1;
    for (int64_t k = 0; k < n; k++) {
        int64_t ok = 1;
        for (int64_t i = 0; i < n; i++) {
            int64_t idx = i + k;
            if (idx >= n) idx -= n;
            if (a[idx] != b[i]) { ok = 0; break; }
        }
        if (ok) return 1;
    }
    return 0;
}

int64_t is_anagram_sorted(const int64_t *a, const int64_t *b, int64_t n) {
    for (int64_t i = 0; i < n; i++) if (a[i] != b[i]) return 0;
    return 1;
}

int64_t longest_run(const int64_t *a, int64_t n) {
    if (n == 0) return 0;
    int64_t best = 1, cur = 1;
    for (int64_t i = 1; i < n; i++) {
        if (a[i] == a[i - 1]) cur++; else cur = 1;
        if (cur > best) best = cur;
    }
    return best;
}

int64_t count_transitions(const int64_t *a, int64_t n) {
    int64_t c = 0;
    for (int64_t i = 1; i < n; i++) if (a[i] != a[i - 1]) c++;
    return c;
}

int64_t first_unique_index(const int64_t *a, int64_t n) {
    for (int64_t i = 0; i < n; i++) {
        int64_t count = 0;
        for (int64_t j = 0; j < n; j++) if (a[j] == a[i]) count++;
        if (count == 1) return i;
    }
    return -1;
}

int64_t palindrome_check(const int64_t *a, int64_t n) {
    int64_t i = 0, j = n - 1;
    while (i < j) {
        if (a[i] != a[j]) return 0;
        i++; j--;
    }
    return 1;
}

int64_t longest_increasing_run(const int64_t *a, int64_t n) {
    if (n == 0) return 0;
    int64_t best = 1, cur = 1;
    for (int64_t i = 1; i < n; i++) {
        if (a[i] > a[i - 1]) cur++; else cur = 1;
        if (cur > best) best = cur;
    }
    return best;
}

int64_t count_distinct_chars(const int64_t *a, int64_t n) {
    if (n == 0) return 0;
    int64_t count = 1;
    for (int64_t i = 1; i < n; i++) if (a[i] != a[i - 1]) count++;
    return count;
}

int64_t max_char_frequency(const int64_t *a, int64_t n) {
    if (n == 0) return 0;
    int64_t best = 1, cur = 1;
    for (int64_t i = 1; i < n; i++) {
        if (a[i] == a[i - 1]) cur++; else cur = 1;
        if (cur > best) best = cur;
    }
    return best;
}

int64_t common_char_count(const int64_t *a, int64_t la, const int64_t *b, int64_t lb) {
    int64_t i = 0, j = 0, count = 0;
    while (i < la && j < lb) {
        if (a[i] == b[j]) { count++; i++; j++; }
        else if (a[i] < b[j]) i++;
        else j++;
    }
    return count;
}

int64_t reverse_equal(const int64_t *a, int64_t n) {
    int64_t i = 0, j = n - 1;
    while (i < j) {
        if (a[i] != a[j]) return 0;
        i++; j--;
    }
    return 1;
}

int64_t run_length_pairs(const int64_t *a, int64_t n) {
    if (n == 0) return 0;
    int64_t pairs = 1;
    for (int64_t i = 1; i < n; i++) if (a[i] != a[i - 1]) pairs++;
    return pairs;
}

int64_t starts_with_arr(const int64_t *text, int64_t tn, const int64_t *pre, int64_t pn) {
    if (pn > tn) return 0;
    for (int64_t i = 0; i < pn; i++) if (text[i] != pre[i]) return 0;
    return 1;
}

int main(void) {
    int64_t kit[6]  = { 107, 105, 116, 116, 101, 110 };
    int64_t sit[7]  = { 115, 105, 116, 116, 105, 110, 103 };
    int64_t r1[5]   = { 97, 98, 99, 100, 101 };
    int64_t r2[5]   = { 97, 98, 122, 100, 101 };
    int64_t rota[6] = { 97, 98, 99, 100, 101, 102 };
    int64_t rotb[6] = { 99, 100, 101, 102, 97, 98 };
    int64_t an[5]   = { 97, 97, 98, 98, 99 };
    int64_t run[6]  = { 1, 1, 1, 2, 2, 3 };
    int64_t fu[6]   = { 1, 2, 2, 3, 1, 4 };
    int64_t pal[5]  = { 1, 2, 3, 2, 1 };
    int64_t inc[5]  = { 1, 2, 3, 1, 2 };
    int64_t sd[6]   = { 1, 1, 2, 3, 3, 3 };
    int64_t c1[5]   = { 1, 2, 2, 3, 5 };
    int64_t c2[4]   = { 2, 2, 3, 4 };
    int64_t occt[5] = { 1, 2, 1, 2, 1 };
    int64_t occp[2] = { 1, 2 };
    int64_t pre[3]  = { 107, 105, 116 };
    printf("edit_distance=%lld\n", (long long)edit_distance(kit, 6, sit, 7));
    printf("lcs_length=%lld\n", (long long)lcs_length(kit, 6, sit, 7));
    printf("hamming_distance=%lld\n", (long long)hamming_distance(r1, r2, 5));
    printf("longest_common_prefix_len=%lld\n", (long long)longest_common_prefix_len(kit, 6, pre, 3));
    printf("count_occurrences=%lld\n", (long long)count_occurrences(occt, 5, occp, 2));
    printf("is_rotation=%lld\n", (long long)is_rotation(rota, rotb, 6));
    printf("is_anagram_sorted=%lld\n", (long long)is_anagram_sorted(an, an, 5));
    printf("longest_run=%lld\n", (long long)longest_run(run, 6));
    printf("count_transitions=%lld\n", (long long)count_transitions(run, 6));
    printf("first_unique_index=%lld\n", (long long)first_unique_index(fu, 6));
    printf("palindrome_check=%lld\n", (long long)palindrome_check(pal, 5));
    printf("longest_increasing_run=%lld\n", (long long)longest_increasing_run(inc, 5));
    printf("count_distinct_chars=%lld\n", (long long)count_distinct_chars(sd, 6));
    printf("max_char_frequency=%lld\n", (long long)max_char_frequency(sd, 6));
    printf("common_char_count=%lld\n", (long long)common_char_count(c1, 5, c2, 4));
    printf("reverse_equal=%lld\n", (long long)reverse_equal(pal, 5));
    printf("run_length_pairs=%lld\n", (long long)run_length_pairs(run, 6));
    printf("starts_with_arr=%lld\n", (long long)starts_with_arr(kit, 6, pre, 3));
    return 0;
}
