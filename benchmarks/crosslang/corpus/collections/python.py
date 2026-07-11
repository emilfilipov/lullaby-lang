"""Cross-language collections suite (Python). Array-as-collection algorithms:
frequency, grouping, and set-like operations over a list and a length. No dicts
or sets are used: everything is counting and scanning, relying on sorted inputs
where noted."""


def count_frequency_of(a: list, n: int, x: int) -> int:
    count = 0
    for i in range(n):
        if a[i] == x:
            count += 1
    return count


def max_frequency(a: list, n: int) -> int:
    if n == 0:
        return 0
    best = 1
    run = 1
    for i in range(1, n):
        run = run + 1 if a[i] == a[i - 1] else 1
        if run > best:
            best = run
    return best


def first_duplicate_value(a: list, n: int) -> int:
    for i in range(1, n):
        if a[i] == a[i - 1]:
            return a[i]
    return -1


def has_pair_sum(a: list, n: int, target: int) -> int:
    lo, hi = 0, n - 1
    while lo < hi:
        s = a[lo] + a[hi]
        if s == target:
            return 1
        elif s < target:
            lo += 1
        else:
            hi -= 1
    return 0


def count_distinct_sorted(a: list, n: int) -> int:
    if n == 0:
        return 0
    count = 1
    for i in range(1, n):
        if a[i] != a[i - 1]:
            count += 1
    return count


def most_common_sorted(a: list, n: int) -> int:
    best_val, best, run = a[0], 1, 1
    for i in range(1, n):
        run = run + 1 if a[i] == a[i - 1] else 1
        if run > best:
            best, best_val = run, a[i]
    return best_val


def count_even(a: list, n: int) -> int:
    count = 0
    for i in range(n):
        if a[i] % 2 == 0:
            count += 1
    return count


def count_odd(a: list, n: int) -> int:
    count = 0
    for i in range(n):
        if a[i] % 2 != 0:
            count += 1
    return count


def partition_point(a: list, n: int) -> int:
    for i in range(n):
        if a[i] >= 0:
            return i
    return n


def count_in_range(a: list, n: int, lo: int, hi: int) -> int:
    count = 0
    for i in range(n):
        if lo <= a[i] <= hi:
            count += 1
    return count


def running_total_last(a: list, n: int) -> int:
    total = 0
    for i in range(n):
        total += a[i]
    return total


def zip_sum(a: list, b: list, n: int) -> int:
    total = 0
    for i in range(n):
        total += a[i] + b[i]
    return total


def intersect_count_sorted(a: list, la: int, b: list, lb: int) -> int:
    i = j = count = 0
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


def union_count_sorted(a: list, la: int, b: list, lb: int) -> int:
    i = j = count = 0
    while i < la and j < lb:
        if a[i] == b[j]:
            count += 1
            i += 1
            j += 1
        elif a[i] < b[j]:
            count += 1
            i += 1
        else:
            count += 1
            j += 1
    count += (la - i) + (lb - j)
    return count


def is_subset_sorted(a: list, la: int, b: list, lb: int) -> int:
    i = j = 0
    while i < la and j < lb:
        if a[i] == b[j]:
            i += 1
            j += 1
        elif a[i] > b[j]:
            j += 1
        else:
            return 0
    return 0 if i < la else 1


def rotate_left_checksum(a: list, n: int, k: int) -> int:
    shift = k % n
    total = 0
    for i in range(n):
        idx = i + shift
        if idx >= n:
            idx -= n
        total += i * a[idx]
    return total


def dedup_sorted_checksum(a: list, n: int) -> int:
    if n == 0:
        return 0
    total, pos, prev = 0, 0, a[0]
    for i in range(1, n):
        if a[i] != prev:
            pos += 1
            total += pos * a[i]
            prev = a[i]
    return total


def chunk_sum_max(a: list, n: int, k: int) -> int:
    window = 0
    for i in range(k):
        window += a[i]
    best = window
    for i in range(k, n):
        window += a[i] - a[i - k]
        if window > best:
            best = window
    return best


def main() -> None:
    a = [-5, -2, -2, 0, 1, 1, 1, 4, 7, 7]
    b = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    c = [1, 3, 5, 7, 9]
    d = [2, 3, 5, 8, 9]
    e = [3, 5, 9]
    print("count_frequency_of=" + str(count_frequency_of(a, 10, 1)))
    print("max_frequency=" + str(max_frequency(a, 10)))
    print("first_duplicate_value=" + str(first_duplicate_value(a, 10)))
    print("has_pair_sum=" + str(has_pair_sum(a, 10, 2)))
    print("count_distinct_sorted=" + str(count_distinct_sorted(a, 10)))
    print("most_common_sorted=" + str(most_common_sorted(a, 10)))
    print("count_even=" + str(count_even(a, 10)))
    print("count_odd=" + str(count_odd(a, 10)))
    print("partition_point=" + str(partition_point(a, 10)))
    print("count_in_range=" + str(count_in_range(a, 10, -2, 1)))
    print("running_total_last=" + str(running_total_last(a, 10)))
    print("zip_sum=" + str(zip_sum(a, b, 10)))
    print("intersect_count_sorted=" + str(intersect_count_sorted(c, 5, d, 5)))
    print("union_count_sorted=" + str(union_count_sorted(c, 5, d, 5)))
    print("is_subset_sorted=" + str(is_subset_sorted(e, 3, d, 5)))
    print("rotate_left_checksum=" + str(rotate_left_checksum(b, 10, 3)))
    print("dedup_sorted_checksum=" + str(dedup_sorted_checksum(a, 10)))
    print("chunk_sum_max=" + str(chunk_sum_max(b, 10, 3)))


if __name__ == "__main__":
    main()
