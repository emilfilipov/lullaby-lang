// Cross-language hashing suite (Rust). Deterministic integer hashes over an
// i64 byte array (values 0..255) plus its length. Arithmetic only — no bitwise
// operators, to stay algorithm-identical to Lullaby. Moduli and products are
// kept below 2^53 so JavaScript doubles stay exact too.

fn djb2_hash(a: &[i64], n: i64) -> i64 {
    let m = 1000000007;
    let mut h = 5381;
    for i in 0..n {
        h = (h * 33 + a[i as usize]) % m;
    }
    h
}

fn fnv1a_arithmetic(a: &[i64], n: i64) -> i64 {
    let m = 100000007;
    let prime = 16777619;
    let mut h = 2166136261i64 % m;
    for i in 0..n {
        h = ((h + a[i as usize]) * prime) % m;
    }
    h
}

fn sum_hash(a: &[i64], n: i64) -> i64 {
    let mut total = 0;
    for i in 0..n {
        total += a[i as usize];
    }
    total
}

fn poly_hash(a: &[i64], n: i64, base: i64, modulus: i64) -> i64 {
    let mut h = 0;
    for i in 0..n {
        h = (h * base + a[i as usize]) % modulus;
    }
    h
}

fn rolling_hash(a: &[i64], n: i64, modulus: i64) -> i64 {
    let mut h = 0;
    for i in 0..n {
        h = (h * 256 + a[i as usize]) % modulus;
    }
    h
}

fn sdbm_hash(a: &[i64], n: i64) -> i64 {
    let m = 1000000007;
    let mut h = 0;
    for i in 0..n {
        h = (a[i as usize] + h * 65599) % m;
    }
    h
}

fn mod_sum_hash(a: &[i64], n: i64, m: i64) -> i64 {
    let mut h = 0;
    for i in 0..n {
        h = (h + a[i as usize]) % m;
    }
    h
}

fn weighted_pos_hash(a: &[i64], n: i64) -> i64 {
    let m = 1000000007;
    let mut h = 0;
    for i in 0..n {
        h = (h + (i + 1) * a[i as usize]) % m;
    }
    h
}

fn xor_free_checksum(a: &[i64], n: i64) -> i64 {
    let mut total = 0;
    for i in 0..n {
        total += a[i as usize];
    }
    total % 256
}

fn product_hash(a: &[i64], n: i64, m: i64) -> i64 {
    let mut h = 1;
    for i in 0..n {
        h = (h * (a[i as usize] + 1)) % m;
    }
    h
}

fn rotate_hash(a: &[i64], n: i64) -> i64 {
    let m = 1073741824;
    let half = 536870912;
    let mut h = 0;
    for i in 0..n {
        let carry = h / half;
        h = (h * 2) % m + carry;
        h = (h + a[i as usize]) % m;
    }
    h
}

fn pearson_like(a: &[i64], n: i64) -> i64 {
    let t = [7, 3, 11, 15, 0, 9, 5, 13, 1, 14, 6, 10, 2, 12, 4, 8];
    let mut h = 0;
    for i in 0..n {
        h = t[((h + a[i as usize]) % 16) as usize];
    }
    h
}

fn length_hash(a: &[i64], n: i64) -> i64 {
    let m = 1000000007;
    if n <= 0 {
        return 0;
    }
    (n * 2654435761 + a[0]) % m
}

fn first_last_hash(a: &[i64], n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    a[0] * 257 + a[(n - 1) as usize]
}

fn midpoint_hash(a: &[i64], n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    a[(n / 2) as usize] * 33 + n
}

fn digit_hash(mut x: i64) -> i64 {
    if x < 0 {
        x = -x;
    }
    if x == 0 {
        return 0;
    }
    let m = 1000000007;
    let mut h = 7;
    while x > 0 {
        h = (h * 31 + x % 10) % m;
        x /= 10;
    }
    h
}

fn pair_hash(a: i64, b: i64) -> i64 {
    let s = a + b;
    s * (s + 1) / 2 + b
}

fn main() {
    let data = [72, 101, 108, 108, 111];
    println!("djb2_hash={}", djb2_hash(&data, 5));
    println!("fnv1a_arithmetic={}", fnv1a_arithmetic(&data, 5));
    println!("sum_hash={}", sum_hash(&data, 5));
    println!("poly_hash={}", poly_hash(&data, 5, 31, 1000000007));
    println!("rolling_hash={}", rolling_hash(&data, 5, 1000000007));
    println!("sdbm_hash={}", sdbm_hash(&data, 5));
    println!("mod_sum_hash={}", mod_sum_hash(&data, 5, 97));
    println!("weighted_pos_hash={}", weighted_pos_hash(&data, 5));
    println!("xor_free_checksum={}", xor_free_checksum(&data, 5));
    println!("product_hash={}", product_hash(&data, 5, 1000000007));
    println!("rotate_hash={}", rotate_hash(&data, 5));
    println!("pearson_like={}", pearson_like(&data, 5));
    println!("length_hash={}", length_hash(&data, 5));
    println!("first_last_hash={}", first_last_hash(&data, 5));
    println!("midpoint_hash={}", midpoint_hash(&data, 5));
    println!("digit_hash={}", digit_hash(1234));
    println!("pair_hash={}", pair_hash(17, 42));
}
