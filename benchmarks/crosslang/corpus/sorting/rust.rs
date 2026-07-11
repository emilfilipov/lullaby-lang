// Cross-language sorting suite (Rust). Classic sort algorithms and order
// statistics over an i64 slice and a length. Each function returns a scalar
// (a checksum sum(i*sorted[i]), a count, or an index). Functions that reorder
// operate on an owned copy so the caller's slice is left untouched.

fn checksum(a: &[i64], n: i64) -> i64 {
    let mut sum = 0;
    for i in 0..n as usize { sum += i as i64 * a[i]; }
    sum
}

fn insertion_sort_checksum(a: &[i64], n: i64) -> i64 {
    let mut buf = a[..n as usize].to_vec();
    for i in 1..n as usize {
        let key = buf[i];
        let mut j = i as i64 - 1;
        while j >= 0 && buf[j as usize] > key {
            buf[(j + 1) as usize] = buf[j as usize];
            j -= 1;
        }
        buf[(j + 1) as usize] = key;
    }
    checksum(&buf, n)
}

fn selection_sort_checksum(a: &[i64], n: i64) -> i64 {
    let mut buf = a[..n as usize].to_vec();
    for i in 0..(n as usize - 1) {
        let mut mi = i;
        for j in (i + 1)..n as usize { if buf[j] < buf[mi] { mi = j; } }
        buf.swap(i, mi);
    }
    checksum(&buf, n)
}

fn bubble_sort_swaps(a: &[i64], n: i64) -> i64 {
    let mut buf = a[..n as usize].to_vec();
    let mut swaps = 0;
    for i in 0..n as usize {
        for j in 0..(n as usize - 1 - i) {
            if buf[j] > buf[j + 1] { buf.swap(j, j + 1); swaps += 1; }
        }
    }
    swaps
}

fn gnome_sort_checksum(a: &[i64], n: i64) -> i64 {
    let mut buf = a[..n as usize].to_vec();
    let mut i = 0usize;
    while i < n as usize {
        if i == 0 || buf[i] >= buf[i - 1] { i += 1; }
        else { buf.swap(i, i - 1); i -= 1; }
    }
    checksum(&buf, n)
}

fn cocktail_sort_checksum(a: &[i64], n: i64) -> i64 {
    let mut buf = a[..n as usize].to_vec();
    let mut lo = 0i64;
    let mut hi = n - 1;
    let mut swapped = true;
    while swapped {
        swapped = false;
        let mut i = lo;
        while i < hi {
            if buf[i as usize] > buf[(i + 1) as usize] { buf.swap(i as usize, (i + 1) as usize); swapped = true; }
            i += 1;
        }
        if !swapped { break; }
        hi -= 1;
        swapped = false;
        let mut k = hi - 1;
        while k >= lo {
            if buf[k as usize] > buf[(k + 1) as usize] { buf.swap(k as usize, (k + 1) as usize); swapped = true; }
            k -= 1;
        }
        lo += 1;
    }
    checksum(&buf, n)
}

fn comb_sort_checksum(a: &[i64], n: i64) -> i64 {
    let mut buf = a[..n as usize].to_vec();
    let mut gap = n;
    let mut swapped = true;
    while gap > 1 || swapped {
        gap = (gap * 10) / 13;
        if gap < 1 { gap = 1; }
        swapped = false;
        let mut i = 0i64;
        while i + gap < n {
            if buf[i as usize] > buf[(i + gap) as usize] { buf.swap(i as usize, (i + gap) as usize); swapped = true; }
            i += 1;
        }
    }
    checksum(&buf, n)
}

fn count_inversions(a: &[i64], n: i64) -> i64 {
    let mut count = 0;
    for i in 0..n as usize {
        for j in (i + 1)..n as usize { if a[i] > a[j] { count += 1; } }
    }
    count
}

fn is_sorted_desc(a: &[i64], n: i64) -> i64 {
    for i in 1..n as usize { if a[i] > a[i - 1] { return 0; } }
    1
}

fn merge_two_sorted_checksum(a: &[i64], la: i64, b: &[i64], lb: i64) -> i64 {
    let (mut i, mut j, mut k, mut sum) = (0i64, 0i64, 0i64, 0i64);
    while i < la && j < lb {
        if a[i as usize] <= b[j as usize] { sum += k * a[i as usize]; i += 1; }
        else { sum += k * b[j as usize]; j += 1; }
        k += 1;
    }
    while i < la { sum += k * a[i as usize]; i += 1; k += 1; }
    while j < lb { sum += k * b[j as usize]; j += 1; k += 1; }
    sum
}

fn partition_lomuto_index(a: &[i64], n: i64) -> i64 {
    let mut buf = a[..n as usize].to_vec();
    let pivot = buf[(n - 1) as usize];
    let mut i = 0usize;
    for j in 0..(n as usize - 1) {
        if buf[j] < pivot { buf.swap(i, j); i += 1; }
    }
    buf.swap(i, (n - 1) as usize);
    i as i64
}

fn kth_smallest(a: &[i64], n: i64, k: i64) -> i64 {
    let mut buf = a[..n as usize].to_vec();
    for i in 0..k as usize {
        let mut mi = i;
        for j in (i + 1)..n as usize { if buf[j] < buf[mi] { mi = j; } }
        buf.swap(i, mi);
    }
    buf[(k - 1) as usize]
}

fn count_sorted_runs(a: &[i64], n: i64) -> i64 {
    if n == 0 { return 0; }
    let mut runs = 1;
    for i in 1..n as usize { if a[i] < a[i - 1] { runs += 1; } }
    runs
}

fn min_swaps_selection(a: &[i64], n: i64) -> i64 {
    let mut buf = a[..n as usize].to_vec();
    let mut swaps = 0;
    for i in 0..(n as usize - 1) {
        let mut mi = i;
        for j in (i + 1)..n as usize { if buf[j] < buf[mi] { mi = j; } }
        if mi != i { buf.swap(i, mi); swaps += 1; }
    }
    swaps
}

fn sorted_median(a: &[i64], n: i64) -> i64 {
    let mut buf = a[..n as usize].to_vec();
    buf.sort();
    buf[(n / 2) as usize]
}

fn sort_evens_first_checksum(a: &[i64], n: i64) -> i64 {
    let mut buf = a[..n as usize].to_vec();
    buf.sort();
    let mut sum = 0i64;
    let mut k = 0i64;
    for i in 0..n as usize {
        if buf[i] % 2 == 0 { sum += k * buf[i]; k += 1; }
    }
    for i in 0..n as usize {
        if buf[i] % 2 != 0 { sum += k * buf[i]; k += 1; }
    }
    sum
}

fn reverse_checksum(a: &[i64], n: i64) -> i64 {
    let mut sum = 0;
    for i in 0..n as usize { sum += i as i64 * a[n as usize - 1 - i]; }
    sum
}

fn max_gap_sorted(a: &[i64], n: i64) -> i64 {
    let mut buf = a[..n as usize].to_vec();
    buf.sort();
    let mut g = 0;
    for i in 1..n as usize {
        let d = buf[i] - buf[i - 1];
        if d > g { g = d; }
    }
    g
}

fn second_smallest(a: &[i64], n: i64) -> i64 {
    let (mut first, mut second) = (a[0], a[1]);
    if second < first { std::mem::swap(&mut first, &mut second); }
    for i in 2..n as usize {
        if a[i] < first { second = first; first = a[i]; }
        else if a[i] < second { second = a[i]; }
    }
    second
}

fn main() {
    let t = [5i64, 3, 8, 1, 9, 2];
    let p = [1i64, 4, 6, 8];
    let q = [2i64, 3, 5, 7, 9];
    println!("insertion_sort_checksum={}", insertion_sort_checksum(&t, 6));
    println!("selection_sort_checksum={}", selection_sort_checksum(&t, 6));
    println!("bubble_sort_swaps={}", bubble_sort_swaps(&t, 6));
    println!("gnome_sort_checksum={}", gnome_sort_checksum(&t, 6));
    println!("cocktail_sort_checksum={}", cocktail_sort_checksum(&t, 6));
    println!("comb_sort_checksum={}", comb_sort_checksum(&t, 6));
    println!("count_inversions={}", count_inversions(&t, 6));
    println!("is_sorted_desc={}", is_sorted_desc(&t, 6));
    println!("merge_two_sorted_checksum={}", merge_two_sorted_checksum(&p, 4, &q, 5));
    println!("partition_lomuto_index={}", partition_lomuto_index(&t, 6));
    println!("kth_smallest={}", kth_smallest(&t, 6, 3));
    println!("count_sorted_runs={}", count_sorted_runs(&t, 6));
    println!("min_swaps_selection={}", min_swaps_selection(&t, 6));
    println!("sorted_median={}", sorted_median(&t, 6));
    println!("sort_evens_first_checksum={}", sort_evens_first_checksum(&t, 6));
    println!("reverse_checksum={}", reverse_checksum(&t, 6));
    println!("max_gap_sorted={}", max_gap_sorted(&t, 6));
    println!("second_smallest={}", second_smallest(&t, 6));
}
