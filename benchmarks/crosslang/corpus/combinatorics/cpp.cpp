// Cross-language combinatorics suite (C++). Counting functions, all int64.
#include <cstdint>
#include <iostream>

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

int main() {
    std::cout << "factorial(10)=" << factorial(10) << "\n";
    std::cout << "permutations_count(10,3)=" << permutations_count(10, 3) << "\n";
    std::cout << "combinations_count(10,3)=" << combinations_count(10, 3) << "\n";
    std::cout << "catalan(10)=" << catalan(10) << "\n";
    std::cout << "pascal_value(10,4)=" << pascal_value(10, 4) << "\n";
    std::cout << "tribonacci(10)=" << tribonacci(10) << "\n";
    std::cout << "lucas(10)=" << lucas(10) << "\n";
    std::cout << "derangement_count(10)=" << derangement_count(10) << "\n";
    std::cout << "bell_number(10)=" << bell_number(10) << "\n";
    std::cout << "stirling_second(10,3)=" << stirling_second(10, 3) << "\n";
    std::cout << "partition_count(40)=" << partition_count(40) << "\n";
    std::cout << "subfactorial(10)=" << subfactorial(10) << "\n";
    std::cout << "binomial_mod(20,10,1000)=" << binomial_mod(20, 10, 1000) << "\n";
    std::cout << "fibonacci_iter(20)=" << fibonacci_iter(20) << "\n";
    std::cout << "padovan(12)=" << padovan(12) << "\n";
    std::cout << "jacobsthal(10)=" << jacobsthal(10) << "\n";
    std::cout << "pentagonal(10)=" << pentagonal(10) << "\n";
    return 0;
}
