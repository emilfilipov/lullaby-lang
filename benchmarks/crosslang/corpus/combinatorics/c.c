/* Cross-language combinatorics suite (C). Counting functions, all int64. */
#include <stdio.h>
#include <stdint.h>

int64_t factorial(int64_t n) {
    int64_t result = 1;
    for (int64_t i = 2; i <= n; i++) {
        result = result * i;
    }
    return result;
}

int64_t permutations_count(int64_t n, int64_t r) {
    int64_t result = 1;
    for (int64_t i = 0; i < r; i++) {
        result = result * (n - i);
    }
    return result;
}

int64_t combinations_count(int64_t n, int64_t r) {
    int64_t result = 1;
    for (int64_t i = 1; i <= r; i++) {
        result = result * (n - r + i) / i;
    }
    return result;
}

int64_t catalan(int64_t n) {
    int64_t result = 1;
    for (int64_t i = 0; i < n; i++) {
        result = result * 2 * (2 * i + 1) / (i + 2);
    }
    return result;
}

int64_t pascal_value(int64_t row, int64_t col) {
    if (col == 0 || col == row) return 1;
    return pascal_value(row - 1, col - 1) + pascal_value(row - 1, col);
}

int64_t tribonacci(int64_t n) {
    if (n == 0) return 0;
    if (n <= 2) return 1;
    int64_t a = 0, b = 1, c = 1;
    for (int64_t i = 3; i <= n; i++) {
        int64_t d = a + b + c;
        a = b;
        b = c;
        c = d;
    }
    return c;
}

int64_t lucas(int64_t n) {
    if (n == 0) return 2;
    if (n == 1) return 1;
    int64_t a = 2, b = 1;
    for (int64_t i = 2; i <= n; i++) {
        int64_t c = a + b;
        a = b;
        b = c;
    }
    return b;
}

int64_t derangement_count(int64_t n) {
    if (n == 0) return 1;
    if (n == 1) return 0;
    int64_t a = 1, b = 0;
    for (int64_t i = 2; i <= n; i++) {
        int64_t c = (i - 1) * (a + b);
        a = b;
        b = c;
    }
    return b;
}

int64_t stirling_second(int64_t n, int64_t k) {
    if (k == 0 && n == 0) return 1;
    if (k == 0 || n == 0) return 0;
    return k * stirling_second(n - 1, k) + stirling_second(n - 1, k - 1);
}

int64_t bell_number(int64_t n) {
    int64_t sum = 0;
    for (int64_t k = 0; k <= n; k++) {
        sum += stirling_second(n, k);
    }
    return sum;
}

int64_t partition_count(int64_t n) {
    int64_t dp[41] = {0};
    dp[0] = 1;
    for (int64_t p = 1; p <= n; p++) {
        for (int64_t j = p; j <= n; j++) {
            dp[j] = dp[j] + dp[j - p];
        }
    }
    return dp[n];
}

int64_t subfactorial(int64_t n) {
    int64_t d = 1, sign = 1;
    for (int64_t i = 1; i <= n; i++) {
        sign = -sign;
        d = i * d + sign;
    }
    return d;
}

int64_t binomial_mod(int64_t n, int64_t r, int64_t m) {
    if (r == 0 || r == n) return 1 % m;
    return (binomial_mod(n - 1, r - 1, m) + binomial_mod(n - 1, r, m)) % m;
}

int64_t fibonacci_iter(int64_t n) {
    if (n == 0) return 0;
    int64_t a = 0, b = 1;
    for (int64_t i = 2; i <= n; i++) {
        int64_t c = a + b;
        a = b;
        b = c;
    }
    return b;
}

int64_t padovan(int64_t n) {
    if (n <= 2) return 1;
    int64_t a = 1, b = 1, c = 1;
    for (int64_t i = 3; i <= n; i++) {
        int64_t d = b + a;
        a = b;
        b = c;
        c = d;
    }
    return c;
}

int64_t jacobsthal(int64_t n) {
    if (n == 0) return 0;
    if (n == 1) return 1;
    int64_t a = 0, b = 1;
    for (int64_t i = 2; i <= n; i++) {
        int64_t c = b + 2 * a;
        a = b;
        b = c;
    }
    return b;
}

int64_t pentagonal(int64_t n) {
    return n * (3 * n - 1) / 2;
}

int main(void) {
    printf("factorial(10)=%lld\n", (long long)factorial(10));
    printf("permutations_count(10,3)=%lld\n", (long long)permutations_count(10, 3));
    printf("combinations_count(10,3)=%lld\n", (long long)combinations_count(10, 3));
    printf("catalan(10)=%lld\n", (long long)catalan(10));
    printf("pascal_value(10,4)=%lld\n", (long long)pascal_value(10, 4));
    printf("tribonacci(10)=%lld\n", (long long)tribonacci(10));
    printf("lucas(10)=%lld\n", (long long)lucas(10));
    printf("derangement_count(10)=%lld\n", (long long)derangement_count(10));
    printf("bell_number(10)=%lld\n", (long long)bell_number(10));
    printf("stirling_second(10,3)=%lld\n", (long long)stirling_second(10, 3));
    printf("partition_count(40)=%lld\n", (long long)partition_count(40));
    printf("subfactorial(10)=%lld\n", (long long)subfactorial(10));
    printf("binomial_mod(20,10,1000)=%lld\n", (long long)binomial_mod(20, 10, 1000));
    printf("fibonacci_iter(20)=%lld\n", (long long)fibonacci_iter(20));
    printf("padovan(12)=%lld\n", (long long)padovan(12));
    printf("jacobsthal(10)=%lld\n", (long long)jacobsthal(10));
    printf("pentagonal(10)=%lld\n", (long long)pentagonal(10));
    return 0;
}
