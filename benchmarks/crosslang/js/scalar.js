// Cross-language scalar function suite (JavaScript). Same 16 hand-written scalar
// algorithms as ../lullaby/scalar.lby, using explicit loops/recursion (no
// library helpers). See ../SPEC.md. Benchmark inputs are non-negative, so
// Math.trunc division and % match the reference's truncating remainder.

function add(a, b) {
  return a + b;
}

function max2(a, b) {
  return a > b ? a : b;
}

function abs_val(n) {
  if (n < 0) {
    return -n;
  }
  return n;
}

function is_even(n) {
  return n % 2 === 0 ? 1 : 0;
}

function clamp(x, lo, hi) {
  if (x < lo) {
    return lo;
  }
  if (x > hi) {
    return hi;
  }
  return x;
}

function sign(n) {
  if (n < 0) {
    return -1;
  }
  if (n > 0) {
    return 1;
  }
  return 0;
}

function factorial(n) {
  let r = 1;
  let i = 2;
  while (i <= n) {
    r *= i;
    i += 1;
  }
  return r;
}

function gcd(a, b) {
  while (b !== 0) {
    const t = a % b;
    a = b;
    b = t;
  }
  return a;
}

function fib_iter(n) {
  let a = 0;
  let b = 1;
  let i = 0;
  while (i < n) {
    const t = a + b;
    a = b;
    b = t;
    i += 1;
  }
  return a;
}

function is_prime(n) {
  if (n < 2) {
    return 0;
  }
  let d = 2;
  while (d * d <= n) {
    if (n % d === 0) {
      return 0;
    }
    d += 1;
  }
  return 1;
}

function int_pow(base, exp) {
  let r = 1;
  let i = 0;
  while (i < exp) {
    r *= base;
    i += 1;
  }
  return r;
}

function collatz_len(n) {
  let steps = 0;
  while (n !== 1) {
    if (n % 2 === 0) {
      n = Math.trunc(n / 2);
    } else {
      n = 3 * n + 1;
    }
    steps += 1;
  }
  return steps;
}

function digit_sum(n) {
  if (n < 0) {
    n = -n;
  }
  let s = 0;
  while (n > 0) {
    s += n % 10;
    n = Math.trunc(n / 10);
  }
  return s;
}

function count_primes_below(n) {
  let count = 0;
  let k = 2;
  while (k < n) {
    if (is_prime(k) === 1) {
      count += 1;
    }
    k += 1;
  }
  return count;
}

function power_mod(base, exp, m) {
  let r = 1;
  base %= m;
  while (exp > 0) {
    if (exp % 2 === 1) {
      r = (r * base) % m;
    }
    exp = Math.trunc(exp / 2);
    base = (base * base) % m;
  }
  return r;
}

function ackermann(m, n) {
  if (m === 0) {
    return n + 1;
  }
  if (n === 0) {
    return ackermann(m - 1, 1);
  }
  return ackermann(m - 1, ackermann(m, n - 1));
}

function main() {
  console.log(add(2, 3));
  console.log(factorial(5));
  console.log(gcd(48, 18));
  console.log(fib_iter(10));
  console.log(count_primes_below(100));
  console.log(power_mod(7, 256, 13));
  console.log(ackermann(2, 3));
}

main();
