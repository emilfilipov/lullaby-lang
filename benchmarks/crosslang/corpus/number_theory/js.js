// Cross-language number theory suite (JavaScript). Classic integer number theory.

function gcd(a, b) {
  while (b !== 0) {
    const t = a % b;
    a = b;
    b = t;
  }
  return a;
}

function int_pow(base, exp) {
  let r = 1;
  for (let i = 0; i < exp; i++) {
    r = r * base;
  }
  return r;
}

function num_digits(n) {
  if (n === 0) {
    return 1;
  }
  let c = 0;
  while (n > 0) {
    c += 1;
    n = Math.trunc(n / 10);
  }
  return c;
}

function lcm(a, b) {
  return Math.trunc(a / gcd(a, b)) * b;
}

function divisor_count(n) {
  let count = 0;
  let d = 1;
  while (d * d <= n) {
    if (n % d === 0) {
      count += 1;
      if (d !== Math.trunc(n / d)) {
        count += 1;
      }
    }
    d += 1;
  }
  return count;
}

function divisor_sum(n) {
  let total = 0;
  let d = 1;
  while (d * d <= n) {
    if (n % d === 0) {
      total += d;
      const other = Math.trunc(n / d);
      if (other !== d) {
        total += other;
      }
    }
    d += 1;
  }
  return total;
}

function is_perfect(n) {
  return divisor_sum(n) - n === n ? 1 : 0;
}

function euler_totient(n) {
  let result = n;
  let p = 2;
  while (p * p <= n) {
    if (n % p === 0) {
      while (n % p === 0) {
        n = Math.trunc(n / p);
      }
      result -= Math.trunc(result / p);
    }
    p += 1;
  }
  if (n > 1) {
    result -= Math.trunc(result / n);
  }
  return result;
}

function count_coprime_below(n) {
  let count = 0;
  for (let k = 1; k < n; k++) {
    if (gcd(k, n) === 1) {
      count += 1;
    }
  }
  return count;
}

function digital_root(n) {
  while (n >= 10) {
    let s = 0;
    while (n > 0) {
      s += n % 10;
      n = Math.trunc(n / 10);
    }
    n = s;
  }
  return n;
}

function is_armstrong(n) {
  const d = num_digits(n);
  let total = 0;
  let m = n;
  while (m > 0) {
    total += int_pow(m % 10, d);
    m = Math.trunc(m / 10);
  }
  return total === n ? 1 : 0;
}

function reverse_digits(n) {
  let r = 0;
  while (n > 0) {
    r = r * 10 + n % 10;
    n = Math.trunc(n / 10);
  }
  return r;
}

function is_palindrome_number(n) {
  return reverse_digits(n) === n ? 1 : 0;
}

function sum_of_squares_digits(n) {
  let total = 0;
  while (n > 0) {
    const d = n % 10;
    total += d * d;
    n = Math.trunc(n / 10);
  }
  return total;
}

function is_happy(n) {
  let steps = 0;
  while (n !== 1 && steps < 1000) {
    n = sum_of_squares_digits(n);
    steps += 1;
  }
  return n === 1 ? 1 : 0;
}

function to_base_digit_sum(n, b) {
  let total = 0;
  while (n > 0) {
    total += n % b;
    n = Math.trunc(n / b);
  }
  return total;
}

function count_trailing_zeros_factorial(n) {
  let count = 0;
  let power = 5;
  while (power <= n) {
    count += Math.trunc(n / power);
    power *= 5;
  }
  return count;
}

function gcd_of_range(lo, hi) {
  let g = lo;
  for (let k = lo; k <= hi; k++) {
    g = gcd(g, k);
  }
  return g;
}

function main() {
  console.log("lcm(12,18)=" + lcm(12, 18));
  console.log("divisor_count(36)=" + divisor_count(36));
  console.log("divisor_sum(36)=" + divisor_sum(36));
  console.log("is_perfect(28)=" + is_perfect(28));
  console.log("euler_totient(36)=" + euler_totient(36));
  console.log("count_coprime_below(36)=" + count_coprime_below(36));
  console.log("digital_root(9875)=" + digital_root(9875));
  console.log("is_armstrong(153)=" + is_armstrong(153));
  console.log("reverse_digits(1234)=" + reverse_digits(1234));
  console.log("is_palindrome_number(1221)=" + is_palindrome_number(1221));
  console.log("sum_of_squares_digits(123)=" + sum_of_squares_digits(123));
  console.log("is_happy(19)=" + is_happy(19));
  console.log("to_base_digit_sum(255,16)=" + to_base_digit_sum(255, 16));
  console.log("count_trailing_zeros_factorial(100)=" + count_trailing_zeros_factorial(100));
  console.log("gcd_of_range(12,24)=" + gcd_of_range(12, 24));
}

main();
