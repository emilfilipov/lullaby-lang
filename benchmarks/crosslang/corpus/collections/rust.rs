// Cross-language collections suite (Rust). Array-as-collection algorithms:
// frequency, grouping, and set-like operations over an i64 slice and a length.
// No hash maps are used: everything is counting and scanning, relying on
// sorted inputs where noted.

fn count_frequency_of(a: &[i64], n: i64, x: i64) -> i64 {
    let mut count = 0;
    for i in 0..n as usize { if a[i] == x { count += 1; } }
    count
}

fn max_frequency(a: &[i64], n: i64) -> i64 {
    if n == 0 { return 0; }
    let (mut best, mut run) = (1, 1);
    for i in 1..n as usize {
        run = if a[i] == a[i - 1] { run + 1 } else { 1 };
        if run > best { best = run; }
    }
    best
}

fn first_duplicate_value(a: &[i64], n: i64) -> i64 {
    for i in 1..n as usize { if a[i] == a[i - 1] { return a[i]; } }
    -1
}

fn has_pair_sum(a: &[i64], n: i64, target: i64) -> i64 {
    let (mut lo, mut hi) = (0i64, n - 1);
    while lo < hi {
        let s = a[lo as usize] + a[hi as usize];
        if s == target { return 1; }
        else if s < target { lo += 1; }
        else { hi -= 1; }
    }
    0
}

fn count_distinct_sorted(a: &[i64], n: i64) -> i64 {
    if n == 0 { return 0; }
    let mut count = 1;
    for i in 1..n as usize { if a[i] != a[i - 1] { count += 1; } }
    count
}

fn most_common_sorted(a: &[i64], n: i64) -> i64 {
    let (mut best_val, mut best, mut run) = (a[0], 1, 1);
    for i in 1..n as usize {
        run = if a[i] == a[i - 1] { run + 1 } else { 1 };
        if run > best { best = run; best_val = a[i]; }
    }
    best_val
}

fn count_even(a: &[i64], n: i64) -> i64 {
    let mut count = 0;
    for i in 0..n as usize { if a[i] % 2 == 0 { count += 1; } }
    count
}

fn count_odd(a: &[i64], n: i64) -> i64 {
    let mut count = 0;
    for i in 0..n as usize { if a[i] % 2 != 0 { count += 1; } }
    count
}

fn partition_point(a: &[i64], n: i64) -> i64 {
    for i in 0..n as usize { if a[i] >= 0 { return i as i64; } }
    n
}

fn count_in_range(a: &[i64], n: i64, lo: i64, hi: i64) -> i64 {
    let mut count = 0;
    for i in 0..n as usize { if a[i] >= lo && a[i] <= hi { count += 1; } }
    count
}

fn running_total_last(a: &[i64], n: i64) -> i64 {
    let mut total = 0;
    for i in 0..n as usize { total += a[i]; }
    total
}

fn zip_sum(a: &[i64], b: &[i64], n: i64) -> i64 {
    let mut sum = 0;
    for i in 0..n as usize { sum += a[i] + b[i]; }
    sum
}

fn intersect_count_sorted(a: &[i64], la: i64, b: &[i64], lb: i64) -> i64 {
    let (mut i, mut j, mut count) = (0usize, 0usize, 0);
    while i < la as usize && j < lb as usize {
        if a[i] == b[j] { count += 1; i += 1; j += 1; }
        else if a[i] < b[j] { i += 1; }
        else { j += 1; }
    }
    count
}

fn union_count_sorted(a: &[i64], la: i64, b: &[i64], lb: i64) -> i64 {
    let (mut i, mut j, mut count) = (0usize, 0usize, 0);
    while i < la as usize && j < lb as usize {
        if a[i] == b[j] { count += 1; i += 1; j += 1; }
        else if a[i] < b[j] { count += 1; i += 1; }
        else { count += 1; j += 1; }
    }
    while i < la as usize { count += 1; i += 1; }
    while j < lb as usize { count += 1; j += 1; }
    count
}

fn is_subset_sorted(a: &[i64], la: i64, b: &[i64], lb: i64) -> i64 {
    let (mut i, mut j) = (0usize, 0usize);
    while i < la as usize && j < lb as usize {
        if a[i] == b[j] { i += 1; j += 1; }
        else if a[i] > b[j] { j += 1; }
        else { return 0; }
    }
    if i < la as usize { 0 } else { 1 }
}

fn rotate_left_checksum(a: &[i64], n: i64, k: i64) -> i64 {
    let shift = k % n;
    let mut sum = 0;
    for i in 0..n {
        let mut idx = i + shift;
        if idx >= n { idx -= n; }
        sum += i * a[idx as usize];
    }
    sum
}

fn dedup_sorted_checksum(a: &[i64], n: i64) -> i64 {
    if n == 0 { return 0; }
    let (mut sum, mut pos, mut prev) = (0, 0, a[0]);
    for i in 1..n as usize {
        if a[i] != prev { pos += 1; sum += pos * a[i]; prev = a[i]; }
    }
    sum
}

fn chunk_sum_max(a: &[i64], n: i64, k: i64) -> i64 {
    let mut window = 0;
    for i in 0..k as usize { window += a[i]; }
    let mut best = window;
    for i in k as usize..n as usize {
        window += a[i] - a[i - k as usize];
        if window > best { best = window; }
    }
    best
}

fn main() {
    let a = [-5i64, -2, -2, 0, 1, 1, 1, 4, 7, 7];
    let b = [1i64, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let c = [1i64, 3, 5, 7, 9];
    let d = [2i64, 3, 5, 8, 9];
    let e = [3i64, 5, 9];
    println!("count_frequency_of={}", count_frequency_of(&a, 10, 1));
    println!("max_frequency={}", max_frequency(&a, 10));
    println!("first_duplicate_value={}", first_duplicate_value(&a, 10));
    println!("has_pair_sum={}", has_pair_sum(&a, 10, 2));
    println!("count_distinct_sorted={}", count_distinct_sorted(&a, 10));
    println!("most_common_sorted={}", most_common_sorted(&a, 10));
    println!("count_even={}", count_even(&a, 10));
    println!("count_odd={}", count_odd(&a, 10));
    println!("partition_point={}", partition_point(&a, 10));
    println!("count_in_range={}", count_in_range(&a, 10, -2, 1));
    println!("running_total_last={}", running_total_last(&a, 10));
    println!("zip_sum={}", zip_sum(&a, &b, 10));
    println!("intersect_count_sorted={}", intersect_count_sorted(&c, 5, &d, 5));
    println!("union_count_sorted={}", union_count_sorted(&c, 5, &d, 5));
    println!("is_subset_sorted={}", is_subset_sorted(&e, 3, &d, 5));
    println!("rotate_left_checksum={}", rotate_left_checksum(&b, 10, 3));
    println!("dedup_sorted_checksum={}", dedup_sorted_checksum(&a, 10));
    println!("chunk_sum_max={}", chunk_sum_max(&b, 10, 3));
}
