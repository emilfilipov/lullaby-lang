// Cross-language validation suite (Rust). Mirrors ../validation/lullaby.lby.
// Arrays are &[i64]; 1-char string args are &str; all predicates return i64 (1/0).
#![allow(dead_code)]

fn luhn_check(digits: &[i64], n: i64) -> i64 {
    let mut sum = 0;
    for i in 0..n {
        let mut d = digits[i as usize];
        let pos = n - 1 - i;
        if pos % 2 == 1 {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
    }
    if sum % 10 == 0 { 1 } else { 0 }
}

fn is_valid_isbn10(digits: &[i64], n: i64) -> i64 {
    if n != 10 {
        return 0;
    }
    let mut sum = 0;
    for i in 0..10 {
        sum += (10 - i) * digits[i as usize];
    }
    if sum % 11 == 0 { 1 } else { 0 }
}

fn in_range(x: i64, lo: i64, hi: i64) -> i64 {
    if x >= lo && x <= hi { 1 } else { 0 }
}

fn is_leap_year(y: i64) -> i64 {
    if y % 400 == 0 {
        1
    } else if y % 100 == 0 {
        0
    } else if y % 4 == 0 {
        1
    } else {
        0
    }
}

fn is_valid_month(m: i64) -> i64 {
    if m >= 1 && m <= 12 { 1 } else { 0 }
}

fn days_in_month(y: i64, m: i64) -> i64 {
    if m == 2 {
        return if is_leap_year(y) == 1 { 29 } else { 28 };
    }
    if m == 4 || m == 6 || m == 9 || m == 11 {
        return 30;
    }
    if m >= 1 && m <= 12 {
        return 31;
    }
    0
}

fn is_valid_day(y: i64, m: i64, d: i64) -> i64 {
    if is_valid_month(m) == 0 {
        return 0;
    }
    if d < 1 || d > days_in_month(y, m) {
        return 0;
    }
    1
}

fn is_ascii_digit(c: &str) -> i64 {
    let b = c.as_bytes()[0];
    if b >= b'0' && b <= b'9' { 1 } else { 0 }
}

fn is_ascii_alpha(c: &str) -> i64 {
    let b = c.as_bytes()[0];
    if (b >= b'A' && b <= b'Z') || (b >= b'a' && b <= b'z') { 1 } else { 0 }
}

fn all_digits(s: &str) -> i64 {
    if s.is_empty() {
        return 0;
    }
    for b in s.bytes() {
        if b < b'0' || b > b'9' {
            return 0;
        }
    }
    1
}

fn password_score(s: &str) -> i64 {
    let mut score = 0;
    if s.len() >= 8 {
        score += 1;
    }
    let mut has_digit = 0;
    let mut has_lower = 0;
    let mut has_upper = 0;
    for b in s.bytes() {
        if b >= b'0' && b <= b'9' {
            has_digit = 1;
        } else if b >= b'a' && b <= b'z' {
            has_lower = 1;
        } else if b >= b'A' && b <= b'Z' {
            has_upper = 1;
        }
    }
    score + has_digit + has_lower + has_upper
}

fn is_hex_string(s: &str) -> i64 {
    if s.is_empty() {
        return 0;
    }
    for b in s.bytes() {
        let ok = (b >= b'0' && b <= b'9') || (b >= b'a' && b <= b'f') || (b >= b'A' && b <= b'F');
        if !ok {
            return 0;
        }
    }
    1
}

fn checksum_mod10(digits: &[i64], n: i64) -> i64 {
    let mut sum = 0;
    for i in 0..n {
        sum += digits[i as usize];
    }
    sum % 10
}

fn valid_percentage(x: i64) -> i64 {
    if x >= 0 && x <= 100 { 1 } else { 0 }
}

fn is_valid_rgb(r: i64, g: i64, b: i64) -> i64 {
    if r >= 0 && r <= 255 && g >= 0 && g <= 255 && b >= 0 && b <= 255 {
        1
    } else {
        0
    }
}

fn even_parity(bits: &[i64], n: i64) -> i64 {
    let mut ones = 0;
    for i in 0..n {
        if bits[i as usize] == 1 {
            ones += 1;
        }
    }
    if ones % 2 == 0 { 1 } else { 0 }
}

fn main() {
    let card = [7, 9, 9, 2, 7, 3, 9, 8, 7, 1, 3];
    let isbn = [0, 3, 0, 6, 4, 0, 6, 1, 5, 2];
    let bits = [1, 0, 1, 1, 0, 0];
    assert_eq!(luhn_check(&card, 11), 1);
    assert_eq!(is_valid_isbn10(&isbn, 10), 1);
    assert_eq!(in_range(5, 1, 10), 1);
    assert_eq!(is_leap_year(2000), 1);
    assert_eq!(is_leap_year(1900), 0);
    assert_eq!(is_valid_month(13), 0);
    assert_eq!(is_valid_day(2021, 2, 29), 0);
    assert_eq!(is_valid_day(2020, 2, 29), 1);
    assert_eq!(is_ascii_digit("7"), 1);
    assert_eq!(is_ascii_alpha("Q"), 1);
    assert_eq!(all_digits("12345"), 1);
    assert_eq!(password_score("Abcdef12"), 4);
    assert_eq!(is_hex_string("1aF"), 1);
    assert_eq!(checksum_mod10(&card, 11), 5);
    assert_eq!(valid_percentage(101), 0);
    assert_eq!(is_valid_rgb(255, 128, 0), 1);
    assert_eq!(even_parity(&bits, 6), 0);
    println!("ok");
}
