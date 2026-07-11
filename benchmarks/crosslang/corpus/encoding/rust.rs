// Cross-language encoding suite (Rust). Checksums, ciphers, bit tricks over
// i64 byte arrays (values 0..255). Arithmetic only — no bitwise operators,
// to stay algorithm-identical to Lullaby.

fn sum_bytes(a: &[i64], n: i64) -> i64 {
    let mut total = 0;
    for i in 0..n {
        total += a[i as usize];
    }
    total
}

fn add_checksum_mod256(a: &[i64], n: i64) -> i64 {
    let mut total = 0;
    for i in 0..n {
        total += a[i as usize];
    }
    total % 256
}

fn fletcher16(a: &[i64], n: i64) -> i64 {
    let mut sum1 = 0;
    let mut sum2 = 0;
    for i in 0..n {
        sum1 = (sum1 + a[i as usize]) % 255;
        sum2 = (sum2 + sum1) % 255;
    }
    sum2 * 256 + sum1
}

fn adler32_small(a: &[i64], n: i64) -> i64 {
    let mut s1 = 1;
    let mut s2 = 0;
    for i in 0..n {
        s1 = (s1 + a[i as usize]) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    s2 * 65536 + s1
}

fn caesar_encrypt_val(c: i64, k: i64) -> i64 {
    if (97..=122).contains(&c) {
        return 97 + (c - 97 + k % 26) % 26;
    }
    c
}

fn caesar_decrypt_val(c: i64, k: i64) -> i64 {
    if (97..=122).contains(&c) {
        return 97 + (c - 97 + 26 - k % 26) % 26;
    }
    c
}

fn rot13_val(c: i64) -> i64 {
    if (97..=122).contains(&c) {
        return 97 + (c - 97 + 13) % 26;
    }
    if (65..=90).contains(&c) {
        return 65 + (c - 65 + 13) % 26;
    }
    c
}

fn count_set_bits(mut x: i64) -> i64 {
    let mut count = 0;
    while x > 0 {
        count += x % 2;
        x /= 2;
    }
    count
}

fn to_binary_length(mut x: i64) -> i64 {
    if x == 0 {
        return 1;
    }
    let mut len = 0;
    while x > 0 {
        len += 1;
        x /= 2;
    }
    len
}

fn hex_digit_value(c: i64) -> i64 {
    if (48..=57).contains(&c) {
        return c - 48;
    }
    if (97..=102).contains(&c) {
        return c - 97 + 10;
    }
    -1
}

fn nibble_to_hex_code(v: i64) -> i64 {
    if v < 10 {
        return 48 + v;
    }
    97 + v - 10
}

fn luhn_from_array(a: &[i64], n: i64) -> i64 {
    let mut sum = 0;
    for i in 0..n {
        let mut d = a[(n - 1 - i) as usize];
        if i % 2 == 1 {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
    }
    if sum % 10 == 0 { 1 } else { 0 }
}

fn parity_bit(a: &[i64], n: i64) -> i64 {
    let mut ones = 0;
    for i in 0..n {
        ones += a[i as usize];
    }
    ones % 2
}

fn crc8_simple(a: &[i64], n: i64) -> i64 {
    let mut crc = 0;
    for i in 0..n {
        crc = (crc + a[i as usize]) % 256;
        crc = (crc * 2) % 256 + crc / 128;
    }
    crc
}

fn digit_product(mut x: i64) -> i64 {
    if x < 0 {
        x = -x;
    }
    if x == 0 {
        return 0;
    }
    let mut p = 1;
    while x > 0 {
        p *= x % 10;
        x /= 10;
    }
    p
}

fn main() {
    let data = [72, 101, 108, 108, 111];
    println!("sum_bytes={}", sum_bytes(&data, 5));
    println!("add_checksum_mod256={}", add_checksum_mod256(&data, 5));
    println!("fletcher16={}", fletcher16(&data, 5));
    println!("adler32_small={}", adler32_small(&data, 5));
    println!("caesar_encrypt_val={}", caesar_encrypt_val(104, 3));
    println!("caesar_decrypt_val={}", caesar_decrypt_val(107, 3));
    println!("rot13_val={}", rot13_val(97));
    println!("count_set_bits={}", count_set_bits(181));
    println!("to_binary_length={}", to_binary_length(181));
    println!("hex_digit_value={}", hex_digit_value(102));
    println!("nibble_to_hex_code={}", nibble_to_hex_code(12));
    let card = [7, 9, 9, 2, 7, 3, 9, 8, 7, 1, 3];
    println!("luhn_from_array={}", luhn_from_array(&card, 11));
    let bits = [1, 0, 1, 1, 0];
    println!("parity_bit={}", parity_bit(&bits, 5));
    println!("crc8_simple={}", crc8_simple(&data, 5));
    println!("digit_product={}", digit_product(1234));
}
