"""Cross-language sorting suite (Python). Classic sort algorithms and order
statistics over a list and a length. Each function returns a scalar (a checksum
sum(i*sorted[i]), a count, or an index). Each function sorts a local copy so the
caller's list is left untouched."""


def checksum(a: list, n: int) -> int:
    return sum(i * a[i] for i in range(n))


def insertion_sort_checksum(a: list, n: int) -> int:
    buf = a[:n]
    for i in range(1, n):
        key = buf[i]
        j = i - 1
        while j >= 0 and buf[j] > key:
            buf[j + 1] = buf[j]
            j -= 1
        buf[j + 1] = key
    return checksum(buf, n)


def selection_sort_checksum(a: list, n: int) -> int:
    buf = a[:n]
    for i in range(n - 1):
        mi = i
        for j in range(i + 1, n):
            if buf[j] < buf[mi]:
                mi = j
        buf[i], buf[mi] = buf[mi], buf[i]
    return checksum(buf, n)


def bubble_sort_swaps(a: list, n: int) -> int:
    buf = a[:n]
    swaps = 0
    for i in range(n):
        for j in range(n - 1 - i):
            if buf[j] > buf[j + 1]:
                buf[j], buf[j + 1] = buf[j + 1], buf[j]
                swaps += 1
    return swaps


def gnome_sort_checksum(a: list, n: int) -> int:
    buf = a[:n]
    i = 0
    while i < n:
        if i == 0 or buf[i] >= buf[i - 1]:
            i += 1
        else:
            buf[i], buf[i - 1] = buf[i - 1], buf[i]
            i -= 1
    return checksum(buf, n)


def cocktail_sort_checksum(a: list, n: int) -> int:
    buf = a[:n]
    lo, hi = 0, n - 1
    swapped = True
    while swapped:
        swapped = False
        for i in range(lo, hi):
            if buf[i] > buf[i + 1]:
                buf[i], buf[i + 1] = buf[i + 1], buf[i]
                swapped = True
        if not swapped:
            break
        hi -= 1
        swapped = False
        for i in range(hi - 1, lo - 1, -1):
            if buf[i] > buf[i + 1]:
                buf[i], buf[i + 1] = buf[i + 1], buf[i]
                swapped = True
        lo += 1
    return checksum(buf, n)


def comb_sort_checksum(a: list, n: int) -> int:
    buf = a[:n]
    gap = n
    swapped = True
    while gap > 1 or swapped:
        gap = (gap * 10) // 13
        if gap < 1:
            gap = 1
        swapped = False
        for i in range(n - gap):
            if buf[i] > buf[i + gap]:
                buf[i], buf[i + gap] = buf[i + gap], buf[i]
                swapped = True
    return checksum(buf, n)


def count_inversions(a: list, n: int) -> int:
    count = 0
    for i in range(n):
        for j in range(i + 1, n):
            if a[i] > a[j]:
                count += 1
    return count


def is_sorted_desc(a: list, n: int) -> int:
    for i in range(1, n):
        if a[i] > a[i - 1]:
            return 0
    return 1


def merge_two_sorted_checksum(a: list, la: int, b: list, lb: int) -> int:
    i = j = k = 0
    total = 0
    while i < la and j < lb:
        if a[i] <= b[j]:
            total += k * a[i]
            i += 1
        else:
            total += k * b[j]
            j += 1
        k += 1
    while i < la:
        total += k * a[i]
        i += 1
        k += 1
    while j < lb:
        total += k * b[j]
        j += 1
        k += 1
    return total


def partition_lomuto_index(a: list, n: int) -> int:
    buf = a[:n]
    pivot = buf[n - 1]
    i = 0
    for j in range(n - 1):
        if buf[j] < pivot:
            buf[i], buf[j] = buf[j], buf[i]
            i += 1
    buf[i], buf[n - 1] = buf[n - 1], buf[i]
    return i


def kth_smallest(a: list, n: int, k: int) -> int:
    buf = a[:n]
    for i in range(k):
        mi = i
        for j in range(i + 1, n):
            if buf[j] < buf[mi]:
                mi = j
        buf[i], buf[mi] = buf[mi], buf[i]
    return buf[k - 1]


def count_sorted_runs(a: list, n: int) -> int:
    if n == 0:
        return 0
    runs = 1
    for i in range(1, n):
        if a[i] < a[i - 1]:
            runs += 1
    return runs


def min_swaps_selection(a: list, n: int) -> int:
    buf = a[:n]
    swaps = 0
    for i in range(n - 1):
        mi = i
        for j in range(i + 1, n):
            if buf[j] < buf[mi]:
                mi = j
        if mi != i:
            buf[i], buf[mi] = buf[mi], buf[i]
            swaps += 1
    return swaps


def sorted_median(a: list, n: int) -> int:
    s = sorted(a[:n])
    return s[n // 2]


def sort_evens_first_checksum(a: list, n: int) -> int:
    s = sorted(a[:n])
    total = 0
    k = 0
    for i in range(n):
        if s[i] % 2 == 0:
            total += k * s[i]
            k += 1
    for i in range(n):
        if s[i] % 2 != 0:
            total += k * s[i]
            k += 1
    return total


def reverse_checksum(a: list, n: int) -> int:
    return sum(i * a[n - 1 - i] for i in range(n))


def max_gap_sorted(a: list, n: int) -> int:
    s = sorted(a[:n])
    g = 0
    for i in range(1, n):
        d = s[i] - s[i - 1]
        if d > g:
            g = d
    return g


def second_smallest(a: list, n: int) -> int:
    first, second = a[0], a[1]
    if second < first:
        first, second = second, first
    for i in range(2, n):
        if a[i] < first:
            second, first = first, a[i]
        elif a[i] < second:
            second = a[i]
    return second


def main() -> None:
    t = [5, 3, 8, 1, 9, 2]
    p = [1, 4, 6, 8]
    q = [2, 3, 5, 7, 9]
    print("insertion_sort_checksum=" + str(insertion_sort_checksum(t, 6)))
    print("selection_sort_checksum=" + str(selection_sort_checksum(t, 6)))
    print("bubble_sort_swaps=" + str(bubble_sort_swaps(t, 6)))
    print("gnome_sort_checksum=" + str(gnome_sort_checksum(t, 6)))
    print("cocktail_sort_checksum=" + str(cocktail_sort_checksum(t, 6)))
    print("comb_sort_checksum=" + str(comb_sort_checksum(t, 6)))
    print("count_inversions=" + str(count_inversions(t, 6)))
    print("is_sorted_desc=" + str(is_sorted_desc(t, 6)))
    print("merge_two_sorted_checksum=" + str(merge_two_sorted_checksum(p, 4, q, 5)))
    print("partition_lomuto_index=" + str(partition_lomuto_index(t, 6)))
    print("kth_smallest=" + str(kth_smallest(t, 6, 3)))
    print("count_sorted_runs=" + str(count_sorted_runs(t, 6)))
    print("min_swaps_selection=" + str(min_swaps_selection(t, 6)))
    print("sorted_median=" + str(sorted_median(t, 6)))
    print("sort_evens_first_checksum=" + str(sort_evens_first_checksum(t, 6)))
    print("reverse_checksum=" + str(reverse_checksum(t, 6)))
    print("max_gap_sorted=" + str(max_gap_sorted(t, 6)))
    print("second_smallest=" + str(second_smallest(t, 6)))


if __name__ == "__main__":
    main()
