"""Cross-language arrays suite (Python). Real-world array/statistics operations
over a list and a length. bubble_sort_checksum sorts a local copy so the
caller's list is left untouched."""


def sum_array(a: list, n: int) -> int:
    total = 0
    for i in range(n):
        total += a[i]
    return total


def max_array(a: list, n: int) -> int:
    m = a[0]
    for i in range(1, n):
        if a[i] > m:
            m = a[i]
    return m


def min_array(a: list, n: int) -> int:
    m = a[0]
    for i in range(1, n):
        if a[i] < m:
            m = a[i]
    return m


def mean_floor(a: list, n: int) -> int:
    return sum_array(a, n) // n


def count_positive(a: list, n: int) -> int:
    count = 0
    for i in range(n):
        if a[i] > 0:
            count += 1
    return count


def count_equal(a: list, n: int, x: int) -> int:
    count = 0
    for i in range(n):
        if a[i] == x:
            count += 1
    return count


def index_of(a: list, n: int, x: int) -> int:
    for i in range(n):
        if a[i] == x:
            return i
    return -1


def binary_search(a: list, n: int, x: int) -> int:
    lo, hi = 0, n - 1
    while lo <= hi:
        mid = (lo + hi) // 2
        if a[mid] == x:
            return mid
        elif a[mid] < x:
            lo = mid + 1
        else:
            hi = mid - 1
    return -1


def is_sorted_asc(a: list, n: int) -> int:
    for i in range(1, n):
        if a[i] < a[i - 1]:
            return 0
    return 1


def range_span(a: list, n: int) -> int:
    return max_array(a, n) - min_array(a, n)


def dot_product(a: list, b: list, n: int) -> int:
    total = 0
    for i in range(n):
        total += a[i] * b[i]
    return total


def count_distinct_sorted(a: list, n: int) -> int:
    if n == 0:
        return 0
    count = 1
    for i in range(1, n):
        if a[i] != a[i - 1]:
            count += 1
    return count


def second_largest(a: list, n: int) -> int:
    first, second = a[0], a[1]
    if second > first:
        first, second = second, first
    for i in range(2, n):
        if a[i] > first:
            second, first = first, a[i]
        elif a[i] > second:
            second = a[i]
    return second


def prefix_sum_last(a: list, n: int) -> int:
    prefix = 0
    for i in range(n):
        prefix += a[i]
    return prefix


def bubble_sort_checksum(a: list, n: int) -> int:
    buf = a[:n]
    for i in range(n):
        for j in range(n - 1 - i):
            if buf[j] > buf[j + 1]:
                buf[j], buf[j + 1] = buf[j + 1], buf[j]
    return sum(i * buf[i] for i in range(n))


def main() -> None:
    t = [5, 3, 8, 1, 9, 2]
    s = [1, 2, 2, 3, 5, 8]
    print("sum_array=" + str(sum_array(t, 6)))
    print("max_array=" + str(max_array(t, 6)))
    print("min_array=" + str(min_array(t, 6)))
    print("mean_floor=" + str(mean_floor(t, 6)))
    print("count_positive=" + str(count_positive(t, 6)))
    print("count_equal=" + str(count_equal(t, 6, 8)))
    print("index_of=" + str(index_of(t, 6, 1)))
    print("binary_search=" + str(binary_search(s, 6, 5)))
    print("is_sorted_asc=" + str(is_sorted_asc(s, 6)))
    print("range_span=" + str(range_span(t, 6)))
    print("dot_product=" + str(dot_product(t, s, 6)))
    print("count_distinct_sorted=" + str(count_distinct_sorted(s, 6)))
    print("second_largest=" + str(second_largest(t, 6)))
    print("prefix_sum_last=" + str(prefix_sum_last(t, 6)))
    print("bubble_sort_checksum=" + str(bubble_sort_checksum(t, 6)))


if __name__ == "__main__":
    main()
