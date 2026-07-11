// Cross-language encoding suite (JavaScript). Checksums, ciphers, bit tricks
// over int byte arrays (values 0..255). Arithmetic only — no bitwise operators,
// to stay algorithm-identical to Lullaby.

function sum_bytes(a, n) {
  let total = 0;
  for (let i = 0; i < n; i++) {
    total += a[i];
  }
  return total;
}

function add_checksum_mod256(a, n) {
  let total = 0;
  for (let i = 0; i < n; i++) {
    total += a[i];
  }
  return total % 256;
}

function fletcher16(a, n) {
  let sum1 = 0;
  let sum2 = 0;
  for (let i = 0; i < n; i++) {
    sum1 = (sum1 + a[i]) % 255;
    sum2 = (sum2 + sum1) % 255;
  }
  return sum2 * 256 + sum1;
}

function adler32_small(a, n) {
  let s1 = 1;
  let s2 = 0;
  for (let i = 0; i < n; i++) {
    s1 = (s1 + a[i]) % 65521;
    s2 = (s2 + s1) % 65521;
  }
  return s2 * 65536 + s1;
}

function caesar_encrypt_val(c, k) {
  if (c >= 97 && c <= 122) {
    return 97 + (c - 97 + (k % 26)) % 26;
  }
  return c;
}

function caesar_decrypt_val(c, k) {
  if (c >= 97 && c <= 122) {
    return 97 + (c - 97 + 26 - (k % 26)) % 26;
  }
  return c;
}

function rot13_val(c) {
  if (c >= 97 && c <= 122) {
    return 97 + (c - 97 + 13) % 26;
  }
  if (c >= 65 && c <= 90) {
    return 65 + (c - 65 + 13) % 26;
  }
  return c;
}

function count_set_bits(x) {
  let count = 0;
  while (x > 0) {
    count += x % 2;
    x = Math.trunc(x / 2);
  }
  return count;
}

function to_binary_length(x) {
  if (x === 0) {
    return 1;
  }
  let length = 0;
  while (x > 0) {
    length += 1;
    x = Math.trunc(x / 2);
  }
  return length;
}

function hex_digit_value(c) {
  if (c >= 48 && c <= 57) {
    return c - 48;
  }
  if (c >= 97 && c <= 102) {
    return c - 97 + 10;
  }
  return -1;
}

function nibble_to_hex_code(v) {
  if (v < 10) {
    return 48 + v;
  }
  return 97 + v - 10;
}

function luhn_from_array(a, n) {
  let total = 0;
  for (let i = 0; i < n; i++) {
    let d = a[n - 1 - i];
    if (i % 2 === 1) {
      d = d * 2;
      if (d > 9) {
        d = d - 9;
      }
    }
    total += d;
  }
  return total % 10 === 0 ? 1 : 0;
}

function parity_bit(a, n) {
  let ones = 0;
  for (let i = 0; i < n; i++) {
    ones += a[i];
  }
  return ones % 2;
}

function crc8_simple(a, n) {
  let crc = 0;
  for (let i = 0; i < n; i++) {
    crc = (crc + a[i]) % 256;
    crc = (crc * 2) % 256 + Math.trunc(crc / 128);
  }
  return crc;
}

function digit_product(x) {
  if (x < 0) {
    x = -x;
  }
  if (x === 0) {
    return 0;
  }
  let p = 1;
  while (x > 0) {
    p = p * (x % 10);
    x = Math.trunc(x / 10);
  }
  return p;
}

function main() {
  const data = [72, 101, 108, 108, 111];
  console.log("sum_bytes=" + sum_bytes(data, 5));
  console.log("add_checksum_mod256=" + add_checksum_mod256(data, 5));
  console.log("fletcher16=" + fletcher16(data, 5));
  console.log("adler32_small=" + adler32_small(data, 5));
  console.log("caesar_encrypt_val=" + caesar_encrypt_val(104, 3));
  console.log("caesar_decrypt_val=" + caesar_decrypt_val(107, 3));
  console.log("rot13_val=" + rot13_val(97));
  console.log("count_set_bits=" + count_set_bits(181));
  console.log("to_binary_length=" + to_binary_length(181));
  console.log("hex_digit_value=" + hex_digit_value(102));
  console.log("nibble_to_hex_code=" + nibble_to_hex_code(12));
  const card = [7, 9, 9, 2, 7, 3, 9, 8, 7, 1, 3];
  console.log("luhn_from_array=" + luhn_from_array(card, 11));
  const bits = [1, 0, 1, 1, 0];
  console.log("parity_bit=" + parity_bit(bits, 5));
  console.log("crc8_simple=" + crc8_simple(data, 5));
  console.log("digit_product=" + digit_product(1234));
}

main();
