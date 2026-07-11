"""Cross-language string-algorithms suite (Python). Classic string algorithms
expressed over lists of i64 character codes, NOT string types, so all six
languages run the identical array algorithm. edit_distance and lcs_length use a
single rolling DP row."""


def edit_distance(a: list, la: int, b: list, lb: int) -> int:
    dp = [0] * 64
    for j in range(lb + 1):
        dp[j] = j
    for i in range(1, la + 1):
        prev = dp[0]
        dp[0] = i
        for j in range(1, lb + 1):
            tmp = dp[j]
            if a[i - 1] == b[j - 1]:
                dp[j] = prev
            else:
                m = dp[j - 1]
                if dp[j] < m:
                    m = dp[j]
                if prev < m:
                    m = prev
                dp[j] = m + 1
            prev = tmp
    return dp[lb]


def lcs_length(a: list, la: int, b: list, lb: int) -> int:
    dp = [0] * 64
    for j in range(lb + 1):
        dp[j] = 0
    for i in range(1, la + 1):
        prev = 0
        for j in range(1, lb + 1):
            tmp = dp[j]
            if a[i - 1] == b[j - 1]:
                dp[j] = prev + 1
            elif dp[j - 1] > dp[j]:
                dp[j] = dp[j - 1]
            prev = tmp
    return dp[lb]


def hamming_distance(a: list, b: list, n: int) -> int:
    d = 0
    for i in range(n):
        if a[i] != b[i]:
            d += 1
    return d


def longest_common_prefix_len(a: list, la: int, b: list, lb: int) -> int:
    m = la if la < lb else lb
    i = 0
    while i < m:
        if a[i] != b[i]:
            return i
        i += 1
    return i


def count_occurrences(text: list, tn: int, pat: list, pn: int) -> int:
    if pn == 0:
        return 0
    count = 0
    i = 0
    while i <= tn - pn:
        ok = 1
        for j in range(pn):
            if text[i + j] != pat[j]:
                ok = 0
                break
        if ok == 1:
            count += 1
        i += 1
    return count


def is_rotation(a: list, b: list, n: int) -> int:
    if n == 0:
        return 1
    for k in range(n):
        ok = 1
        for i in range(n):
            idx = i + k
            if idx >= n:
                idx -= n
            if a[idx] != b[i]:
                ok = 0
                break
        if ok == 1:
            return 1
    return 0


def is_anagram_sorted(a: list, b: list, n: int) -> int:
    for i in range(n):
        if a[i] != b[i]:
            return 0
    return 1


def longest_run(a: list, n: int) -> int:
    if n == 0:
        return 0
    best, cur = 1, 1
    for i in range(1, n):
        cur = cur + 1 if a[i] == a[i - 1] else 1
        if cur > best:
            best = cur
    return best


def count_transitions(a: list, n: int) -> int:
    c = 0
    for i in range(1, n):
        if a[i] != a[i - 1]:
            c += 1
    return c


def first_unique_index(a: list, n: int) -> int:
    for i in range(n):
        count = 0
        for j in range(n):
            if a[j] == a[i]:
                count += 1
        if count == 1:
            return i
    return -1


def palindrome_check(a: list, n: int) -> int:
    i, j = 0, n - 1
    while i < j:
        if a[i] != a[j]:
            return 0
        i += 1
        j -= 1
    return 1


def longest_increasing_run(a: list, n: int) -> int:
    if n == 0:
        return 0
    best, cur = 1, 1
    for i in range(1, n):
        cur = cur + 1 if a[i] > a[i - 1] else 1
        if cur > best:
            best = cur
    return best


def count_distinct_chars(a: list, n: int) -> int:
    if n == 0:
        return 0
    count = 1
    for i in range(1, n):
        if a[i] != a[i - 1]:
            count += 1
    return count


def max_char_frequency(a: list, n: int) -> int:
    if n == 0:
        return 0
    best, cur = 1, 1
    for i in range(1, n):
        cur = cur + 1 if a[i] == a[i - 1] else 1
        if cur > best:
            best = cur
    return best


def common_char_count(a: list, la: int, b: list, lb: int) -> int:
    i, j, count = 0, 0, 0
    while i < la and j < lb:
        if a[i] == b[j]:
            count += 1
            i += 1
            j += 1
        elif a[i] < b[j]:
            i += 1
        else:
            j += 1
    return count


def reverse_equal(a: list, n: int) -> int:
    i, j = 0, n - 1
    while i < j:
        if a[i] != a[j]:
            return 0
        i += 1
        j -= 1
    return 1


def run_length_pairs(a: list, n: int) -> int:
    if n == 0:
        return 0
    pairs = 1
    for i in range(1, n):
        if a[i] != a[i - 1]:
            pairs += 1
    return pairs


def starts_with_arr(text: list, tn: int, pre: list, pn: int) -> int:
    if pn > tn:
        return 0
    for i in range(pn):
        if text[i] != pre[i]:
            return 0
    return 1


def main() -> None:
    kit = [107, 105, 116, 116, 101, 110]
    sit = [115, 105, 116, 116, 105, 110, 103]
    r1 = [97, 98, 99, 100, 101]
    r2 = [97, 98, 122, 100, 101]
    rota = [97, 98, 99, 100, 101, 102]
    rotb = [99, 100, 101, 102, 97, 98]
    an = [97, 97, 98, 98, 99]
    run = [1, 1, 1, 2, 2, 3]
    fu = [1, 2, 2, 3, 1, 4]
    pal = [1, 2, 3, 2, 1]
    inc = [1, 2, 3, 1, 2]
    sd = [1, 1, 2, 3, 3, 3]
    c1 = [1, 2, 2, 3, 5]
    c2 = [2, 2, 3, 4]
    occt = [1, 2, 1, 2, 1]
    occp = [1, 2]
    pre = [107, 105, 116]
    print("edit_distance=" + str(edit_distance(kit, 6, sit, 7)))
    print("lcs_length=" + str(lcs_length(kit, 6, sit, 7)))
    print("hamming_distance=" + str(hamming_distance(r1, r2, 5)))
    print("longest_common_prefix_len=" + str(longest_common_prefix_len(kit, 6, pre, 3)))
    print("count_occurrences=" + str(count_occurrences(occt, 5, occp, 2)))
    print("is_rotation=" + str(is_rotation(rota, rotb, 6)))
    print("is_anagram_sorted=" + str(is_anagram_sorted(an, an, 5)))
    print("longest_run=" + str(longest_run(run, 6)))
    print("count_transitions=" + str(count_transitions(run, 6)))
    print("first_unique_index=" + str(first_unique_index(fu, 6)))
    print("palindrome_check=" + str(palindrome_check(pal, 5)))
    print("longest_increasing_run=" + str(longest_increasing_run(inc, 5)))
    print("count_distinct_chars=" + str(count_distinct_chars(sd, 6)))
    print("max_char_frequency=" + str(max_char_frequency(sd, 6)))
    print("common_char_count=" + str(common_char_count(c1, 5, c2, 4)))
    print("reverse_equal=" + str(reverse_equal(pal, 5)))
    print("run_length_pairs=" + str(run_length_pairs(run, 6)))
    print("starts_with_arr=" + str(starts_with_arr(kit, 6, pre, 3)))


if __name__ == "__main__":
    main()
