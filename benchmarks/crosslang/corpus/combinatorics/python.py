# Cross-language combinatorics suite (Python). Counting functions, all integers.


def factorial(n):
    result = 1
    for i in range(2, n + 1):
        result = result * i
    return result


def permutations_count(n, r):
    result = 1
    for i in range(r):
        result = result * (n - i)
    return result


def combinations_count(n, r):
    result = 1
    for i in range(1, r + 1):
        result = result * (n - r + i) // i
    return result


def catalan(n):
    result = 1
    for i in range(n):
        result = result * 2 * (2 * i + 1) // (i + 2)
    return result


def pascal_value(row, col):
    if col == 0 or col == row:
        return 1
    return pascal_value(row - 1, col - 1) + pascal_value(row - 1, col)


def tribonacci(n):
    if n == 0:
        return 0
    if n <= 2:
        return 1
    a, b, c = 0, 1, 1
    for _ in range(3, n + 1):
        a, b, c = b, c, a + b + c
    return c


def lucas(n):
    if n == 0:
        return 2
    if n == 1:
        return 1
    a, b = 2, 1
    for _ in range(2, n + 1):
        a, b = b, a + b
    return b


def derangement_count(n):
    if n == 0:
        return 1
    if n == 1:
        return 0
    a, b = 1, 0
    for i in range(2, n + 1):
        a, b = b, (i - 1) * (a + b)
    return b


def stirling_second(n, k):
    if k == 0 and n == 0:
        return 1
    if k == 0 or n == 0:
        return 0
    return k * stirling_second(n - 1, k) + stirling_second(n - 1, k - 1)


def bell_number(n):
    total = 0
    for k in range(n + 1):
        total += stirling_second(n, k)
    return total


def partition_count(n):
    dp = [0] * (n + 1)
    dp[0] = 1
    for p in range(1, n + 1):
        for j in range(p, n + 1):
            dp[j] = dp[j] + dp[j - p]
    return dp[n]


def subfactorial(n):
    d = 1
    sign = 1
    for i in range(1, n + 1):
        sign = -sign
        d = i * d + sign
    return d


def binomial_mod(n, r, m):
    if r == 0 or r == n:
        return 1 % m
    return (binomial_mod(n - 1, r - 1, m) + binomial_mod(n - 1, r, m)) % m


def fibonacci_iter(n):
    if n == 0:
        return 0
    a, b = 0, 1
    for _ in range(2, n + 1):
        a, b = b, a + b
    return b


def padovan(n):
    if n <= 2:
        return 1
    a, b, c = 1, 1, 1
    for _ in range(3, n + 1):
        a, b, c = b, c, b + a
    return c


def jacobsthal(n):
    if n == 0:
        return 0
    if n == 1:
        return 1
    a, b = 0, 1
    for _ in range(2, n + 1):
        a, b = b, b + 2 * a
    return b


def pentagonal(n):
    return n * (3 * n - 1) // 2


def main():
    print("factorial(10)=" + str(factorial(10)))
    print("permutations_count(10,3)=" + str(permutations_count(10, 3)))
    print("combinations_count(10,3)=" + str(combinations_count(10, 3)))
    print("catalan(10)=" + str(catalan(10)))
    print("pascal_value(10,4)=" + str(pascal_value(10, 4)))
    print("tribonacci(10)=" + str(tribonacci(10)))
    print("lucas(10)=" + str(lucas(10)))
    print("derangement_count(10)=" + str(derangement_count(10)))
    print("bell_number(10)=" + str(bell_number(10)))
    print("stirling_second(10,3)=" + str(stirling_second(10, 3)))
    print("partition_count(40)=" + str(partition_count(40)))
    print("subfactorial(10)=" + str(subfactorial(10)))
    print("binomial_mod(20,10,1000)=" + str(binomial_mod(20, 10, 1000)))
    print("fibonacci_iter(20)=" + str(fibonacci_iter(20)))
    print("padovan(12)=" + str(padovan(12)))
    print("jacobsthal(10)=" + str(jacobsthal(10)))
    print("pentagonal(10)=" + str(pentagonal(10)))


if __name__ == "__main__":
    main()
