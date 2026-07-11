// Cross-language arrays suite (Rust). Real-world array/statistics operations
// over an i64 slice and a length. bubble_sort_checksum sorts an owned copy so
// the caller's slice is left untouched.

fn sum_array(a: &[i64], n: i64) -> i64 {
    let mut sum = 0;
    for i in 0..n as usize { sum += a[i]; }
    sum
}

fn max_array(a: &[i64], n: i64) -> i64 {
    let mut m = a[0];
    for i in 1..n as usize { if a[i] > m { m = a[i]; } }
    m
}

fn min_array(a: &[i64], n: i64) -> i64 {
    let mut m = a[0];
    for i in 1..n as usize { if a[i] < m { m = a[i]; } }
    m
}

fn mean_floor(a: &[i64], n: i64) -> i64 {
    sum_array(a, n) / n
}

fn count_positive(a: &[i64], n: i64) -> i64 {
    let mut count = 0;
    for i in 0..n as usize { if a[i] > 0 { count += 1; } }
    count
}

fn count_equal(a: &[i64], n: i64, x: i64) -> i64 {
    let mut count = 0;
    for i in 0..n as usize { if a[i] == x { count += 1; } }
    count
}

fn index_of(a: &[i64], n: i64, x: i64) -> i64 {
    for i in 0..n as usize { if a[i] == x { return i as i64; } }
    -1
}

fn binary_search(a: &[i64], n: i64, x: i64) -> i64 {
    let mut lo = 0i64;
    let mut hi = n - 1;
    while lo <= hi {
        let mid = (lo + hi) / 2;
        if a[mid as usize] == x { return mid; }
        else if a[mid as usize] < x { lo = mid + 1; }
        else { hi = mid - 1; }
    }
    -1
}

fn is_sorted_asc(a: &[i64], n: i64) -> i64 {
    for i in 1..n as usize { if a[i] < a[i - 1] { return 0; } }
    1
}

fn range_span(a: &[i64], n: i64) -> i64 {
    max_array(a, n) - min_array(a, n)
}

fn dot_product(a: &[i64], b: &[i64], n: i64) -> i64 {
    let mut sum = 0;
    for i in 0..n as usize { sum += a[i] * b[i]; }
    sum
}

fn count_distinct_sorted(a: &[i64], n: i64) -> i64 {
    if n == 0 { return 0; }
    let mut count = 1;
    for i in 1..n as usize { if a[i] != a[i - 1] { count += 1; } }
    count
}

fn second_largest(a: &[i64], n: i64) -> i64 {
    let (mut first, mut second) = (a[0], a[1]);
    if second > first { std::mem::swap(&mut first, &mut second); }
    for i in 2..n as usize {
        if a[i] > first { second = first; first = a[i]; }
        else if a[i] > second { second = a[i]; }
    }
    second
}

fn prefix_sum_last(a: &[i64], n: i64) -> i64 {
    let mut prefix = 0;
    for i in 0..n as usize { prefix += a[i]; }
    prefix
}

fn bubble_sort_checksum(a: &[i64], n: i64) -> i64 {
    let mut buf = a[..n as usize].to_vec();
    for i in 0..n as usize {
        for j in 0..(n as usize - 1 - i) {
            if buf[j] > buf[j + 1] { buf.swap(j, j + 1); }
        }
    }
    let mut sum = 0;
    for i in 0..n as usize { sum += i as i64 * buf[i]; }
    sum
}

fn main() {
    let t = [5i64, 3, 8, 1, 9, 2];
    let s = [1i64, 2, 2, 3, 5, 8];
    println!("sum_array={}", sum_array(&t, 6));
    println!("max_array={}", max_array(&t, 6));
    println!("min_array={}", min_array(&t, 6));
    println!("mean_floor={}", mean_floor(&t, 6));
    println!("count_positive={}", count_positive(&t, 6));
    println!("count_equal={}", count_equal(&t, 6, 8));
    println!("index_of={}", index_of(&t, 6, 1));
    println!("binary_search={}", binary_search(&s, 6, 5));
    println!("is_sorted_asc={}", is_sorted_asc(&s, 6));
    println!("range_span={}", range_span(&t, 6));
    println!("dot_product={}", dot_product(&t, &s, 6));
    println!("count_distinct_sorted={}", count_distinct_sorted(&s, 6));
    println!("second_largest={}", second_largest(&t, 6));
    println!("prefix_sum_last={}", prefix_sum_last(&t, 6));
    println!("bubble_sort_checksum={}", bubble_sort_checksum(&t, 6));
}
