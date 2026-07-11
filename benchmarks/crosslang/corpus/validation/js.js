// Cross-language validation suite (JavaScript). Mirrors ../validation/lullaby.lby.
// Arrays are number[]; 1-char string args are strings; all predicates return int (1/0).

const assert = require("assert");

function luhn_check(digits, n) {
  let total = 0;
  for (let i = 0; i < n; i++) {
    let d = digits[i];
    const pos = n - 1 - i;
    if (pos % 2 === 1) {
      d *= 2;
      if (d > 9) d -= 9;
    }
    total += d;
  }
  return total % 10 === 0 ? 1 : 0;
}

function is_valid_isbn10(digits, n) {
  if (n !== 10) return 0;
  let total = 0;
  for (let i = 0; i < 10; i++) total += (10 - i) * digits[i];
  return total % 11 === 0 ? 1 : 0;
}

function in_range(x, lo, hi) {
  return lo <= x && x <= hi ? 1 : 0;
}

function is_leap_year(y) {
  if (y % 400 === 0) return 1;
  if (y % 100 === 0) return 0;
  return y % 4 === 0 ? 1 : 0;
}

function is_valid_month(m) {
  return m >= 1 && m <= 12 ? 1 : 0;
}

function days_in_month(y, m) {
  if (m === 2) return is_leap_year(y) ? 29 : 28;
  if (m === 4 || m === 6 || m === 9 || m === 11) return 30;
  if (m >= 1 && m <= 12) return 31;
  return 0;
}

function is_valid_day(y, m, d) {
  if (!is_valid_month(m)) return 0;
  if (d < 1 || d > days_in_month(y, m)) return 0;
  return 1;
}

function is_ascii_digit(c) {
  const b = c.charCodeAt(0);
  return b >= 48 && b <= 57 ? 1 : 0;
}

function is_ascii_alpha(c) {
  const b = c.charCodeAt(0);
  return (b >= 65 && b <= 90) || (b >= 97 && b <= 122) ? 1 : 0;
}

function all_digits(s) {
  if (s.length === 0) return 0;
  for (let i = 0; i < s.length; i++) {
    const b = s.charCodeAt(i);
    if (b < 48 || b > 57) return 0;
  }
  return 1;
}

function password_score(s) {
  let score = 0;
  if (s.length >= 8) score += 1;
  let has_digit = 0;
  let has_lower = 0;
  let has_upper = 0;
  for (let i = 0; i < s.length; i++) {
    const b = s.charCodeAt(i);
    if (b >= 48 && b <= 57) has_digit = 1;
    else if (b >= 97 && b <= 122) has_lower = 1;
    else if (b >= 65 && b <= 90) has_upper = 1;
  }
  return score + has_digit + has_lower + has_upper;
}

function is_hex_string(s) {
  if (s.length === 0) return 0;
  for (let i = 0; i < s.length; i++) {
    const b = s.charCodeAt(i);
    const ok = (b >= 48 && b <= 57) || (b >= 97 && b <= 102) || (b >= 65 && b <= 70);
    if (!ok) return 0;
  }
  return 1;
}

function checksum_mod10(digits, n) {
  let sum = 0;
  for (let i = 0; i < n; i++) sum += digits[i];
  return sum % 10;
}

function valid_percentage(x) {
  return x >= 0 && x <= 100 ? 1 : 0;
}

function is_valid_rgb(r, g, b) {
  return r >= 0 && r <= 255 && g >= 0 && g <= 255 && b >= 0 && b <= 255 ? 1 : 0;
}

function even_parity(bits, n) {
  let ones = 0;
  for (let i = 0; i < n; i++) if (bits[i] === 1) ones += 1;
  return ones % 2 === 0 ? 1 : 0;
}

function main() {
  const card = [7, 9, 9, 2, 7, 3, 9, 8, 7, 1, 3];
  const isbn = [0, 3, 0, 6, 4, 0, 6, 1, 5, 2];
  const bits = [1, 0, 1, 1, 0, 0];
  assert.strictEqual(luhn_check(card, 11), 1);
  assert.strictEqual(is_valid_isbn10(isbn, 10), 1);
  assert.strictEqual(in_range(5, 1, 10), 1);
  assert.strictEqual(is_leap_year(2000), 1);
  assert.strictEqual(is_leap_year(1900), 0);
  assert.strictEqual(is_valid_month(13), 0);
  assert.strictEqual(is_valid_day(2021, 2, 29), 0);
  assert.strictEqual(is_valid_day(2020, 2, 29), 1);
  assert.strictEqual(is_ascii_digit("7"), 1);
  assert.strictEqual(is_ascii_alpha("Q"), 1);
  assert.strictEqual(all_digits("12345"), 1);
  assert.strictEqual(password_score("Abcdef12"), 4);
  assert.strictEqual(is_hex_string("1aF"), 1);
  assert.strictEqual(checksum_mod10(card, 11), 5);
  assert.strictEqual(valid_percentage(101), 0);
  assert.strictEqual(is_valid_rgb(255, 128, 0), 1);
  assert.strictEqual(even_parity(bits, 6), 0);
  console.log("ok");
}

main();
