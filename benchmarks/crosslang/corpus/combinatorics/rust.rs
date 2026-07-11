// Cross-language combinatorics suite (Rust). Counting functions, all i64.

fn factorial(n: i64) -> i64 {
    let mut result: i64 = 1;
    for i in 2..=n {
        result *= i;
    }
    result
}

fn permutations_count(n: i64, r: i64) -> i64 {
    let mut result: i64 = 1;
    for i in 0..r {
        result *= n - i;
    }
    result
}

fn combinations_count(n: i64, r: i64) -> i64 {
    let mut result: i64 = 1;
    for i in 1..=r {
        result = result * (n - r + i) / i;
    }
    result
}

fn catalan(n: i64) -> i64 {
    let mut result: i64 = 1;
    for i in 0..n {
        result = result * 2 * (2 * i + 1) / (i + 2);
    }
    result
}

fn pascal_value(row: i64, col: i64) -> i64 {
    if col == 0 || col == row {
        return 1;
    }
    pascal_value(row - 1, col - 1) + pascal_value(row - 1, col)
}

fn tribonacci(n: i64) -> i64 {
    if n == 0 {
        return 0;
    }
    if n <= 2 {
        return 1;
    }
    let (mut a, mut b, mut c) = (0, 1, 1);
    for _ in 3..=n {
        let d = a + b + c;
        a = b;
        b = c;
        c = d;
    }
    c
}

fn lucas(n: i64) -> i64 {
    if n == 0 {
        return 2;
    }
    if n == 1 {
        return 1;
    }
    let (mut a, mut b) = (2, 1);
    for _ in 2..=n {
        let c = a + b;
        a = b;
        b = c;
    }
    b
}

fn derangement_count(n: i64) -> i64 {
    if n == 0 {
        return 1;
    }
    if n == 1 {
        return 0;
    }
    let (mut a, mut b) = (1, 0);
    for i in 2..=n {
        let c = (i - 1) * (a + b);
        a = b;
        b = c;
    }
    b
}

fn stirling_second(n: i64, k: i64) -> i64 {
    if k == 0 && n == 0 {
        return 1;
    }
    if k == 0 || n == 0 {
        return 0;
    }
    k * stirling_second(n - 1, k) + stirling_second(n - 1, k - 1)
}

fn bell_number(n: i64) -> i64 {
    let mut sum = 0;
    for k in 0..=n {
        sum += stirling_second(n, k);
    }
    sum
}

fn partition_count(n: i64) -> i64 {
    let mut dp = [0i64; 41];
    dp[0] = 1;
    for p in 1..=n {
        for j in p..=n {
            dp[j as usize] += dp[(j - p) as usize];
        }
    }
    dp[n as usize]
}

fn subfactorial(n: i64) -> i64 {
    let mut d = 1;
    let mut sign = 1;
    for i in 1..=n {
        sign = -sign;
        d = i * d + sign;
    }
    d
}

fn binomial_mod(n: i64, r: i64, m: i64) -> i64 {
    if r == 0 || r == n {
        return 1 % m;
    }
    (binomial_mod(n - 1, r - 1, m) + binomial_mod(n - 1, r, m)) % m
}

fn fibonacci_iter(n: i64) -> i64 {
    if n == 0 {
        return 0;
    }
    let (mut a, mut b) = (0, 1);
    for _ in 2..=n {
        let c = a + b;
        a = b;
        b = c;
    }
    b
}

fn padovan(n: i64) -> i64 {
    if n <= 2 {
        return 1;
    }
    let (mut a, mut b, mut c) = (1, 1, 1);
    for _ in 3..=n {
        let d = b + a;
        a = b;
        b = c;
        c = d;
    }
    c
}

fn jacobsthal(n: i64) -> i64 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    let (mut a, mut b) = (0, 1);
    for _ in 2..=n {
        let c = b + 2 * a;
        a = b;
        b = c;
    }
    b
}

fn pentagonal(n: i64) -> i64 {
    n * (3 * n - 1) / 2
}

fn main() {
    println!("factorial(10)={}", factorial(10));
    println!("permutations_count(10,3)={}", permutations_count(10, 3));
    println!("combinations_count(10,3)={}", combinations_count(10, 3));
    println!("catalan(10)={}", catalan(10));
    println!("pascal_value(10,4)={}", pascal_value(10, 4));
    println!("tribonacci(10)={}", tribonacci(10));
    println!("lucas(10)={}", lucas(10));
    println!("derangement_count(10)={}", derangement_count(10));
    println!("bell_number(10)={}", bell_number(10));
    println!("stirling_second(10,3)={}", stirling_second(10, 3));
    println!("partition_count(40)={}", partition_count(40));
    println!("subfactorial(10)={}", subfactorial(10));
    println!("binomial_mod(20,10,1000)={}", binomial_mod(20, 10, 1000));
    println!("fibonacci_iter(20)={}", fibonacci_iter(20));
    println!("padovan(12)={}", padovan(12));
    println!("jacobsthal(10)={}", jacobsthal(10));
    println!("pentagonal(10)={}", pentagonal(10));
}
