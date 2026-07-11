// Cross-language string-algorithms suite (Rust). Classic string algorithms
// expressed over i64 slices of character codes, NOT string types, so all six
// languages run the identical array algorithm. edit_distance and lcs_length use
// a single rolling DP row.

fn edit_distance(a: &[i64], la: i64, b: &[i64], lb: i64) -> i64 {
    let mut dp = [0i64; 64];
    for j in 0..=lb as usize { dp[j] = j as i64; }
    for i in 1..=la as usize {
        let mut prev = dp[0];
        dp[0] = i as i64;
        for j in 1..=lb as usize {
            let tmp = dp[j];
            if a[i - 1] == b[j - 1] {
                dp[j] = prev;
            } else {
                let mut m = dp[j - 1];
                if dp[j] < m { m = dp[j]; }
                if prev < m { m = prev; }
                dp[j] = m + 1;
            }
            prev = tmp;
        }
    }
    dp[lb as usize]
}

fn lcs_length(a: &[i64], la: i64, b: &[i64], lb: i64) -> i64 {
    let mut dp = [0i64; 64];
    for j in 0..=lb as usize { dp[j] = 0; }
    for i in 1..=la as usize {
        let mut prev = 0i64;
        for j in 1..=lb as usize {
            let tmp = dp[j];
            if a[i - 1] == b[j - 1] {
                dp[j] = prev + 1;
            } else if dp[j - 1] > dp[j] {
                dp[j] = dp[j - 1];
            }
            prev = tmp;
        }
    }
    dp[lb as usize]
}

fn hamming_distance(a: &[i64], b: &[i64], n: i64) -> i64 {
    let mut d = 0;
    for i in 0..n as usize { if a[i] != b[i] { d += 1; } }
    d
}

fn longest_common_prefix_len(a: &[i64], la: i64, b: &[i64], lb: i64) -> i64 {
    let m = if la < lb { la } else { lb };
    let mut i = 0i64;
    while i < m {
        if a[i as usize] != b[i as usize] { return i; }
        i += 1;
    }
    i
}

fn count_occurrences(text: &[i64], tn: i64, pat: &[i64], pn: i64) -> i64 {
    if pn == 0 { return 0; }
    let mut count = 0;
    let mut i = 0i64;
    while i <= tn - pn {
        let mut ok = 1;
        for j in 0..pn as usize {
            if text[i as usize + j] != pat[j] { ok = 0; break; }
        }
        if ok == 1 { count += 1; }
        i += 1;
    }
    count
}

fn is_rotation(a: &[i64], b: &[i64], n: i64) -> i64 {
    if n == 0 { return 1; }
    for k in 0..n {
        let mut ok = 1;
        for i in 0..n {
            let mut idx = i + k;
            if idx >= n { idx -= n; }
            if a[idx as usize] != b[i as usize] { ok = 0; break; }
        }
        if ok == 1 { return 1; }
    }
    0
}

fn is_anagram_sorted(a: &[i64], b: &[i64], n: i64) -> i64 {
    for i in 0..n as usize { if a[i] != b[i] { return 0; } }
    1
}

fn longest_run(a: &[i64], n: i64) -> i64 {
    if n == 0 { return 0; }
    let (mut best, mut cur) = (1i64, 1i64);
    for i in 1..n as usize {
        if a[i] == a[i - 1] { cur += 1; } else { cur = 1; }
        if cur > best { best = cur; }
    }
    best
}

fn count_transitions(a: &[i64], n: i64) -> i64 {
    let mut c = 0;
    for i in 1..n as usize { if a[i] != a[i - 1] { c += 1; } }
    c
}

fn first_unique_index(a: &[i64], n: i64) -> i64 {
    for i in 0..n as usize {
        let mut count = 0;
        for j in 0..n as usize { if a[j] == a[i] { count += 1; } }
        if count == 1 { return i as i64; }
    }
    -1
}

fn palindrome_check(a: &[i64], n: i64) -> i64 {
    let (mut i, mut j) = (0i64, n - 1);
    while i < j {
        if a[i as usize] != a[j as usize] { return 0; }
        i += 1; j -= 1;
    }
    1
}

fn longest_increasing_run(a: &[i64], n: i64) -> i64 {
    if n == 0 { return 0; }
    let (mut best, mut cur) = (1i64, 1i64);
    for i in 1..n as usize {
        if a[i] > a[i - 1] { cur += 1; } else { cur = 1; }
        if cur > best { best = cur; }
    }
    best
}

fn count_distinct_chars(a: &[i64], n: i64) -> i64 {
    if n == 0 { return 0; }
    let mut count = 1;
    for i in 1..n as usize { if a[i] != a[i - 1] { count += 1; } }
    count
}

fn max_char_frequency(a: &[i64], n: i64) -> i64 {
    if n == 0 { return 0; }
    let (mut best, mut cur) = (1i64, 1i64);
    for i in 1..n as usize {
        if a[i] == a[i - 1] { cur += 1; } else { cur = 1; }
        if cur > best { best = cur; }
    }
    best
}

fn common_char_count(a: &[i64], la: i64, b: &[i64], lb: i64) -> i64 {
    let (mut i, mut j, mut count) = (0i64, 0i64, 0i64);
    while i < la && j < lb {
        if a[i as usize] == b[j as usize] { count += 1; i += 1; j += 1; }
        else if a[i as usize] < b[j as usize] { i += 1; }
        else { j += 1; }
    }
    count
}

fn reverse_equal(a: &[i64], n: i64) -> i64 {
    let (mut i, mut j) = (0i64, n - 1);
    while i < j {
        if a[i as usize] != a[j as usize] { return 0; }
        i += 1; j -= 1;
    }
    1
}

fn run_length_pairs(a: &[i64], n: i64) -> i64 {
    if n == 0 { return 0; }
    let mut pairs = 1;
    for i in 1..n as usize { if a[i] != a[i - 1] { pairs += 1; } }
    pairs
}

fn starts_with_arr(text: &[i64], tn: i64, pre: &[i64], pn: i64) -> i64 {
    if pn > tn { return 0; }
    for i in 0..pn as usize { if text[i] != pre[i] { return 0; } }
    1
}

fn main() {
    let kit  = [107i64, 105, 116, 116, 101, 110];
    let sit  = [115i64, 105, 116, 116, 105, 110, 103];
    let r1   = [97i64, 98, 99, 100, 101];
    let r2   = [97i64, 98, 122, 100, 101];
    let rota = [97i64, 98, 99, 100, 101, 102];
    let rotb = [99i64, 100, 101, 102, 97, 98];
    let an   = [97i64, 97, 98, 98, 99];
    let run  = [1i64, 1, 1, 2, 2, 3];
    let fu   = [1i64, 2, 2, 3, 1, 4];
    let pal  = [1i64, 2, 3, 2, 1];
    let inc  = [1i64, 2, 3, 1, 2];
    let sd   = [1i64, 1, 2, 3, 3, 3];
    let c1   = [1i64, 2, 2, 3, 5];
    let c2   = [2i64, 2, 3, 4];
    let occt = [1i64, 2, 1, 2, 1];
    let occp = [1i64, 2];
    let pre  = [107i64, 105, 116];
    println!("edit_distance={}", edit_distance(&kit, 6, &sit, 7));
    println!("lcs_length={}", lcs_length(&kit, 6, &sit, 7));
    println!("hamming_distance={}", hamming_distance(&r1, &r2, 5));
    println!("longest_common_prefix_len={}", longest_common_prefix_len(&kit, 6, &pre, 3));
    println!("count_occurrences={}", count_occurrences(&occt, 5, &occp, 2));
    println!("is_rotation={}", is_rotation(&rota, &rotb, 6));
    println!("is_anagram_sorted={}", is_anagram_sorted(&an, &an, 5));
    println!("longest_run={}", longest_run(&run, 6));
    println!("count_transitions={}", count_transitions(&run, 6));
    println!("first_unique_index={}", first_unique_index(&fu, 6));
    println!("palindrome_check={}", palindrome_check(&pal, 5));
    println!("longest_increasing_run={}", longest_increasing_run(&inc, 5));
    println!("count_distinct_chars={}", count_distinct_chars(&sd, 6));
    println!("max_char_frequency={}", max_char_frequency(&sd, 6));
    println!("common_char_count={}", common_char_count(&c1, 5, &c2, 4));
    println!("reverse_equal={}", reverse_equal(&pal, 5));
    println!("run_length_pairs={}", run_length_pairs(&run, 6));
    println!("starts_with_arr={}", starts_with_arr(&kit, 6, &pre, 3));
}
