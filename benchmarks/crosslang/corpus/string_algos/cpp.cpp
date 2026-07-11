// Cross-language string-algorithms suite (C++). Classic string algorithms
// expressed over i64 ARRAYS of character codes, NOT string types, so all six
// languages run the identical array algorithm. edit_distance and lcs_length use
// a single rolling DP row.
#include <cstdint>
#include <iostream>

std::int64_t edit_distance(const std::int64_t *a, std::int64_t la, const std::int64_t *b, std::int64_t lb) {
    std::int64_t dp[64];
    for (std::int64_t j = 0; j <= lb; j++) dp[j] = j;
    for (std::int64_t i = 1; i <= la; i++) {
        std::int64_t prev = dp[0];
        dp[0] = i;
        for (std::int64_t j = 1; j <= lb; j++) {
            std::int64_t tmp = dp[j];
            if (a[i - 1] == b[j - 1]) {
                dp[j] = prev;
            } else {
                std::int64_t m = dp[j - 1];
                if (dp[j] < m) m = dp[j];
                if (prev < m) m = prev;
                dp[j] = m + 1;
            }
            prev = tmp;
        }
    }
    return dp[lb];
}

std::int64_t lcs_length(const std::int64_t *a, std::int64_t la, const std::int64_t *b, std::int64_t lb) {
    std::int64_t dp[64];
    for (std::int64_t j = 0; j <= lb; j++) dp[j] = 0;
    for (std::int64_t i = 1; i <= la; i++) {
        std::int64_t prev = 0;
        for (std::int64_t j = 1; j <= lb; j++) {
            std::int64_t tmp = dp[j];
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

std::int64_t hamming_distance(const std::int64_t *a, const std::int64_t *b, std::int64_t n) {
    std::int64_t d = 0;
    for (std::int64_t i = 0; i < n; i++) if (a[i] != b[i]) d++;
    return d;
}

std::int64_t longest_common_prefix_len(const std::int64_t *a, std::int64_t la, const std::int64_t *b, std::int64_t lb) {
    std::int64_t m = la < lb ? la : lb;
    std::int64_t i = 0;
    while (i < m) {
        if (a[i] != b[i]) return i;
        i++;
    }
    return i;
}

std::int64_t count_occurrences(const std::int64_t *text, std::int64_t tn, const std::int64_t *pat, std::int64_t pn) {
    if (pn == 0) return 0;
    std::int64_t count = 0;
    for (std::int64_t i = 0; i <= tn - pn; i++) {
        std::int64_t ok = 1;
        for (std::int64_t j = 0; j < pn; j++) {
            if (text[i + j] != pat[j]) { ok = 0; break; }
        }
        if (ok) count++;
    }
    return count;
}

std::int64_t is_rotation(const std::int64_t *a, const std::int64_t *b, std::int64_t n) {
    if (n == 0) return 1;
    for (std::int64_t k = 0; k < n; k++) {
        std::int64_t ok = 1;
        for (std::int64_t i = 0; i < n; i++) {
            std::int64_t idx = i + k;
            if (idx >= n) idx -= n;
            if (a[idx] != b[i]) { ok = 0; break; }
        }
        if (ok) return 1;
    }
    return 0;
}

std::int64_t is_anagram_sorted(const std::int64_t *a, const std::int64_t *b, std::int64_t n) {
    for (std::int64_t i = 0; i < n; i++) if (a[i] != b[i]) return 0;
    return 1;
}

std::int64_t longest_run(const std::int64_t *a, std::int64_t n) {
    if (n == 0) return 0;
    std::int64_t best = 1, cur = 1;
    for (std::int64_t i = 1; i < n; i++) {
        if (a[i] == a[i - 1]) cur++; else cur = 1;
        if (cur > best) best = cur;
    }
    return best;
}

std::int64_t count_transitions(const std::int64_t *a, std::int64_t n) {
    std::int64_t c = 0;
    for (std::int64_t i = 1; i < n; i++) if (a[i] != a[i - 1]) c++;
    return c;
}

std::int64_t first_unique_index(const std::int64_t *a, std::int64_t n) {
    for (std::int64_t i = 0; i < n; i++) {
        std::int64_t count = 0;
        for (std::int64_t j = 0; j < n; j++) if (a[j] == a[i]) count++;
        if (count == 1) return i;
    }
    return -1;
}

std::int64_t palindrome_check(const std::int64_t *a, std::int64_t n) {
    std::int64_t i = 0, j = n - 1;
    while (i < j) {
        if (a[i] != a[j]) return 0;
        i++; j--;
    }
    return 1;
}

std::int64_t longest_increasing_run(const std::int64_t *a, std::int64_t n) {
    if (n == 0) return 0;
    std::int64_t best = 1, cur = 1;
    for (std::int64_t i = 1; i < n; i++) {
        if (a[i] > a[i - 1]) cur++; else cur = 1;
        if (cur > best) best = cur;
    }
    return best;
}

std::int64_t count_distinct_chars(const std::int64_t *a, std::int64_t n) {
    if (n == 0) return 0;
    std::int64_t count = 1;
    for (std::int64_t i = 1; i < n; i++) if (a[i] != a[i - 1]) count++;
    return count;
}

std::int64_t max_char_frequency(const std::int64_t *a, std::int64_t n) {
    if (n == 0) return 0;
    std::int64_t best = 1, cur = 1;
    for (std::int64_t i = 1; i < n; i++) {
        if (a[i] == a[i - 1]) cur++; else cur = 1;
        if (cur > best) best = cur;
    }
    return best;
}

std::int64_t common_char_count(const std::int64_t *a, std::int64_t la, const std::int64_t *b, std::int64_t lb) {
    std::int64_t i = 0, j = 0, count = 0;
    while (i < la && j < lb) {
        if (a[i] == b[j]) { count++; i++; j++; }
        else if (a[i] < b[j]) i++;
        else j++;
    }
    return count;
}

std::int64_t reverse_equal(const std::int64_t *a, std::int64_t n) {
    std::int64_t i = 0, j = n - 1;
    while (i < j) {
        if (a[i] != a[j]) return 0;
        i++; j--;
    }
    return 1;
}

std::int64_t run_length_pairs(const std::int64_t *a, std::int64_t n) {
    if (n == 0) return 0;
    std::int64_t pairs = 1;
    for (std::int64_t i = 1; i < n; i++) if (a[i] != a[i - 1]) pairs++;
    return pairs;
}

std::int64_t starts_with_arr(const std::int64_t *text, std::int64_t tn, const std::int64_t *pre, std::int64_t pn) {
    if (pn > tn) return 0;
    for (std::int64_t i = 0; i < pn; i++) if (text[i] != pre[i]) return 0;
    return 1;
}

int main() {
    std::int64_t kit[6]  = { 107, 105, 116, 116, 101, 110 };
    std::int64_t sit[7]  = { 115, 105, 116, 116, 105, 110, 103 };
    std::int64_t r1[5]   = { 97, 98, 99, 100, 101 };
    std::int64_t r2[5]   = { 97, 98, 122, 100, 101 };
    std::int64_t rota[6] = { 97, 98, 99, 100, 101, 102 };
    std::int64_t rotb[6] = { 99, 100, 101, 102, 97, 98 };
    std::int64_t an[5]   = { 97, 97, 98, 98, 99 };
    std::int64_t run[6]  = { 1, 1, 1, 2, 2, 3 };
    std::int64_t fu[6]   = { 1, 2, 2, 3, 1, 4 };
    std::int64_t pal[5]  = { 1, 2, 3, 2, 1 };
    std::int64_t inc[5]  = { 1, 2, 3, 1, 2 };
    std::int64_t sd[6]   = { 1, 1, 2, 3, 3, 3 };
    std::int64_t c1[5]   = { 1, 2, 2, 3, 5 };
    std::int64_t c2[4]   = { 2, 2, 3, 4 };
    std::int64_t occt[5] = { 1, 2, 1, 2, 1 };
    std::int64_t occp[2] = { 1, 2 };
    std::int64_t pre[3]  = { 107, 105, 116 };
    std::cout << "edit_distance=" << edit_distance(kit, 6, sit, 7) << "\n";
    std::cout << "lcs_length=" << lcs_length(kit, 6, sit, 7) << "\n";
    std::cout << "hamming_distance=" << hamming_distance(r1, r2, 5) << "\n";
    std::cout << "longest_common_prefix_len=" << longest_common_prefix_len(kit, 6, pre, 3) << "\n";
    std::cout << "count_occurrences=" << count_occurrences(occt, 5, occp, 2) << "\n";
    std::cout << "is_rotation=" << is_rotation(rota, rotb, 6) << "\n";
    std::cout << "is_anagram_sorted=" << is_anagram_sorted(an, an, 5) << "\n";
    std::cout << "longest_run=" << longest_run(run, 6) << "\n";
    std::cout << "count_transitions=" << count_transitions(run, 6) << "\n";
    std::cout << "first_unique_index=" << first_unique_index(fu, 6) << "\n";
    std::cout << "palindrome_check=" << palindrome_check(pal, 5) << "\n";
    std::cout << "longest_increasing_run=" << longest_increasing_run(inc, 5) << "\n";
    std::cout << "count_distinct_chars=" << count_distinct_chars(sd, 6) << "\n";
    std::cout << "max_char_frequency=" << max_char_frequency(sd, 6) << "\n";
    std::cout << "common_char_count=" << common_char_count(c1, 5, c2, 4) << "\n";
    std::cout << "reverse_equal=" << reverse_equal(pal, 5) << "\n";
    std::cout << "run_length_pairs=" << run_length_pairs(run, 6) << "\n";
    std::cout << "starts_with_arr=" << starts_with_arr(kit, 6, pre, 3) << "\n";
    return 0;
}
