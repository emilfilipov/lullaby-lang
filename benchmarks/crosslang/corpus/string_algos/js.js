// Cross-language string-algorithms suite (JavaScript). Classic string
// algorithms expressed over arrays of integer character codes, NOT string
// types, so all six languages run the identical array algorithm. edit_distance
// and lcs_length use a single rolling DP row.

function edit_distance(a, la, b, lb) {
    const dp = new Array(64).fill(0);
    for (let j = 0; j <= lb; j++) dp[j] = j;
    for (let i = 1; i <= la; i++) {
        let prev = dp[0];
        dp[0] = i;
        for (let j = 1; j <= lb; j++) {
            const tmp = dp[j];
            if (a[i - 1] === b[j - 1]) {
                dp[j] = prev;
            } else {
                let m = dp[j - 1];
                if (dp[j] < m) m = dp[j];
                if (prev < m) m = prev;
                dp[j] = m + 1;
            }
            prev = tmp;
        }
    }
    return dp[lb];
}

function lcs_length(a, la, b, lb) {
    const dp = new Array(64).fill(0);
    for (let j = 0; j <= lb; j++) dp[j] = 0;
    for (let i = 1; i <= la; i++) {
        let prev = 0;
        for (let j = 1; j <= lb; j++) {
            const tmp = dp[j];
            if (a[i - 1] === b[j - 1]) {
                dp[j] = prev + 1;
            } else if (dp[j - 1] > dp[j]) {
                dp[j] = dp[j - 1];
            }
            prev = tmp;
        }
    }
    return dp[lb];
}

function hamming_distance(a, b, n) {
    let d = 0;
    for (let i = 0; i < n; i++) if (a[i] !== b[i]) d++;
    return d;
}

function longest_common_prefix_len(a, la, b, lb) {
    const m = la < lb ? la : lb;
    let i = 0;
    while (i < m) {
        if (a[i] !== b[i]) return i;
        i++;
    }
    return i;
}

function count_occurrences(text, tn, pat, pn) {
    if (pn === 0) return 0;
    let count = 0;
    let i = 0;
    while (i <= tn - pn) {
        let ok = 1;
        for (let j = 0; j < pn; j++) {
            if (text[i + j] !== pat[j]) { ok = 0; break; }
        }
        if (ok === 1) count++;
        i++;
    }
    return count;
}

function is_rotation(a, b, n) {
    if (n === 0) return 1;
    for (let k = 0; k < n; k++) {
        let ok = 1;
        for (let i = 0; i < n; i++) {
            let idx = i + k;
            if (idx >= n) idx -= n;
            if (a[idx] !== b[i]) { ok = 0; break; }
        }
        if (ok === 1) return 1;
    }
    return 0;
}

function is_anagram_sorted(a, b, n) {
    for (let i = 0; i < n; i++) if (a[i] !== b[i]) return 0;
    return 1;
}

function longest_run(a, n) {
    if (n === 0) return 0;
    let best = 1, cur = 1;
    for (let i = 1; i < n; i++) {
        cur = a[i] === a[i - 1] ? cur + 1 : 1;
        if (cur > best) best = cur;
    }
    return best;
}

function count_transitions(a, n) {
    let c = 0;
    for (let i = 1; i < n; i++) if (a[i] !== a[i - 1]) c++;
    return c;
}

function first_unique_index(a, n) {
    for (let i = 0; i < n; i++) {
        let count = 0;
        for (let j = 0; j < n; j++) if (a[j] === a[i]) count++;
        if (count === 1) return i;
    }
    return -1;
}

function palindrome_check(a, n) {
    let i = 0, j = n - 1;
    while (i < j) {
        if (a[i] !== a[j]) return 0;
        i++; j--;
    }
    return 1;
}

function longest_increasing_run(a, n) {
    if (n === 0) return 0;
    let best = 1, cur = 1;
    for (let i = 1; i < n; i++) {
        cur = a[i] > a[i - 1] ? cur + 1 : 1;
        if (cur > best) best = cur;
    }
    return best;
}

function count_distinct_chars(a, n) {
    if (n === 0) return 0;
    let count = 1;
    for (let i = 1; i < n; i++) if (a[i] !== a[i - 1]) count++;
    return count;
}

function max_char_frequency(a, n) {
    if (n === 0) return 0;
    let best = 1, cur = 1;
    for (let i = 1; i < n; i++) {
        cur = a[i] === a[i - 1] ? cur + 1 : 1;
        if (cur > best) best = cur;
    }
    return best;
}

function common_char_count(a, la, b, lb) {
    let i = 0, j = 0, count = 0;
    while (i < la && j < lb) {
        if (a[i] === b[j]) { count++; i++; j++; }
        else if (a[i] < b[j]) i++;
        else j++;
    }
    return count;
}

function reverse_equal(a, n) {
    let i = 0, j = n - 1;
    while (i < j) {
        if (a[i] !== a[j]) return 0;
        i++; j--;
    }
    return 1;
}

function run_length_pairs(a, n) {
    if (n === 0) return 0;
    let pairs = 1;
    for (let i = 1; i < n; i++) if (a[i] !== a[i - 1]) pairs++;
    return pairs;
}

function starts_with_arr(text, tn, pre, pn) {
    if (pn > tn) return 0;
    for (let i = 0; i < pn; i++) if (text[i] !== pre[i]) return 0;
    return 1;
}

function main() {
    const kit = [107, 105, 116, 116, 101, 110];
    const sit = [115, 105, 116, 116, 105, 110, 103];
    const r1 = [97, 98, 99, 100, 101];
    const r2 = [97, 98, 122, 100, 101];
    const rota = [97, 98, 99, 100, 101, 102];
    const rotb = [99, 100, 101, 102, 97, 98];
    const an = [97, 97, 98, 98, 99];
    const run = [1, 1, 1, 2, 2, 3];
    const fu = [1, 2, 2, 3, 1, 4];
    const pal = [1, 2, 3, 2, 1];
    const inc = [1, 2, 3, 1, 2];
    const sd = [1, 1, 2, 3, 3, 3];
    const c1 = [1, 2, 2, 3, 5];
    const c2 = [2, 2, 3, 4];
    const occt = [1, 2, 1, 2, 1];
    const occp = [1, 2];
    const pre = [107, 105, 116];
    console.log("edit_distance=" + edit_distance(kit, 6, sit, 7));
    console.log("lcs_length=" + lcs_length(kit, 6, sit, 7));
    console.log("hamming_distance=" + hamming_distance(r1, r2, 5));
    console.log("longest_common_prefix_len=" + longest_common_prefix_len(kit, 6, pre, 3));
    console.log("count_occurrences=" + count_occurrences(occt, 5, occp, 2));
    console.log("is_rotation=" + is_rotation(rota, rotb, 6));
    console.log("is_anagram_sorted=" + is_anagram_sorted(an, an, 5));
    console.log("longest_run=" + longest_run(run, 6));
    console.log("count_transitions=" + count_transitions(run, 6));
    console.log("first_unique_index=" + first_unique_index(fu, 6));
    console.log("palindrome_check=" + palindrome_check(pal, 5));
    console.log("longest_increasing_run=" + longest_increasing_run(inc, 5));
    console.log("count_distinct_chars=" + count_distinct_chars(sd, 6));
    console.log("max_char_frequency=" + max_char_frequency(sd, 6));
    console.log("common_char_count=" + common_char_count(c1, 5, c2, 4));
    console.log("reverse_equal=" + reverse_equal(pal, 5));
    console.log("run_length_pairs=" + run_length_pairs(run, 6));
    console.log("starts_with_arr=" + starts_with_arr(kit, 6, pre, 3));
}

main();
