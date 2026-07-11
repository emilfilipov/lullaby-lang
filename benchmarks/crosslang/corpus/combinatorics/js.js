// Cross-language combinatorics suite (JavaScript). Counting functions.
// JS numbers are doubles; inputs are kept small so all results are exact.

function factorial(n) {
    let result = 1;
    for (let i = 2; i <= n; i++) {
        result = result * i;
    }
    return result;
}

function permutations_count(n, r) {
    let result = 1;
    for (let i = 0; i < r; i++) {
        result = result * (n - i);
    }
    return result;
}

function combinations_count(n, r) {
    let result = 1;
    for (let i = 1; i <= r; i++) {
        result = result * (n - r + i) / i;
    }
    return result;
}

function catalan(n) {
    let result = 1;
    for (let i = 0; i < n; i++) {
        result = result * 2 * (2 * i + 1) / (i + 2);
    }
    return result;
}

function pascal_value(row, col) {
    if (col === 0 || col === row) return 1;
    return pascal_value(row - 1, col - 1) + pascal_value(row - 1, col);
}

function tribonacci(n) {
    if (n === 0) return 0;
    if (n <= 2) return 1;
    let a = 0, b = 1, c = 1;
    for (let i = 3; i <= n; i++) {
        const d = a + b + c;
        a = b;
        b = c;
        c = d;
    }
    return c;
}

function lucas(n) {
    if (n === 0) return 2;
    if (n === 1) return 1;
    let a = 2, b = 1;
    for (let i = 2; i <= n; i++) {
        const c = a + b;
        a = b;
        b = c;
    }
    return b;
}

function derangement_count(n) {
    if (n === 0) return 1;
    if (n === 1) return 0;
    let a = 1, b = 0;
    for (let i = 2; i <= n; i++) {
        const c = (i - 1) * (a + b);
        a = b;
        b = c;
    }
    return b;
}

function stirling_second(n, k) {
    if (k === 0 && n === 0) return 1;
    if (k === 0 || n === 0) return 0;
    return k * stirling_second(n - 1, k) + stirling_second(n - 1, k - 1);
}

function bell_number(n) {
    let sum = 0;
    for (let k = 0; k <= n; k++) {
        sum += stirling_second(n, k);
    }
    return sum;
}

function partition_count(n) {
    const dp = new Array(n + 1).fill(0);
    dp[0] = 1;
    for (let p = 1; p <= n; p++) {
        for (let j = p; j <= n; j++) {
            dp[j] = dp[j] + dp[j - p];
        }
    }
    return dp[n];
}

function subfactorial(n) {
    let d = 1, sign = 1;
    for (let i = 1; i <= n; i++) {
        sign = -sign;
        d = i * d + sign;
    }
    return d;
}

function binomial_mod(n, r, m) {
    if (r === 0 || r === n) return 1 % m;
    return (binomial_mod(n - 1, r - 1, m) + binomial_mod(n - 1, r, m)) % m;
}

function fibonacci_iter(n) {
    if (n === 0) return 0;
    let a = 0, b = 1;
    for (let i = 2; i <= n; i++) {
        const c = a + b;
        a = b;
        b = c;
    }
    return b;
}

function padovan(n) {
    if (n <= 2) return 1;
    let a = 1, b = 1, c = 1;
    for (let i = 3; i <= n; i++) {
        const d = b + a;
        a = b;
        b = c;
        c = d;
    }
    return c;
}

function jacobsthal(n) {
    if (n === 0) return 0;
    if (n === 1) return 1;
    let a = 0, b = 1;
    for (let i = 2; i <= n; i++) {
        const c = b + 2 * a;
        a = b;
        b = c;
    }
    return b;
}

function pentagonal(n) {
    return n * (3 * n - 1) / 2;
}

function main() {
    console.log("factorial(10)=" + factorial(10));
    console.log("permutations_count(10,3)=" + permutations_count(10, 3));
    console.log("combinations_count(10,3)=" + combinations_count(10, 3));
    console.log("catalan(10)=" + catalan(10));
    console.log("pascal_value(10,4)=" + pascal_value(10, 4));
    console.log("tribonacci(10)=" + tribonacci(10));
    console.log("lucas(10)=" + lucas(10));
    console.log("derangement_count(10)=" + derangement_count(10));
    console.log("bell_number(10)=" + bell_number(10));
    console.log("stirling_second(10,3)=" + stirling_second(10, 3));
    console.log("partition_count(40)=" + partition_count(40));
    console.log("subfactorial(10)=" + subfactorial(10));
    console.log("binomial_mod(20,10,1000)=" + binomial_mod(20, 10, 1000));
    console.log("fibonacci_iter(20)=" + fibonacci_iter(20));
    console.log("padovan(12)=" + padovan(12));
    console.log("jacobsthal(10)=" + jacobsthal(10));
    console.log("pentagonal(10)=" + pentagonal(10));
}

main();
