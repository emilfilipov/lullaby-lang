// Cross-language matrix suite (Rust). Dense matrix operations over a flat
// row-major i64 slice: an n*n matrix is passed as a slice plus its order n,
// with element (r, c) stored at m[r * n + c].

fn diagonal_sum_main(m: &[i64], n: i64) -> i64 {
    let mut sum = 0;
    for i in 0..n { sum += m[(i * n + i) as usize]; }
    sum
}

fn trace(m: &[i64], n: i64) -> i64 {
    diagonal_sum_main(m, n)
}

fn diagonal_sum_anti(m: &[i64], n: i64) -> i64 {
    let mut sum = 0;
    for i in 0..n { sum += m[(i * n + (n - 1 - i)) as usize]; }
    sum
}

fn row_sum(m: &[i64], n: i64, r: i64) -> i64 {
    let mut sum = 0;
    for c in 0..n { sum += m[(r * n + c) as usize]; }
    sum
}

fn col_sum(m: &[i64], n: i64, c: i64) -> i64 {
    let mut sum = 0;
    for r in 0..n { sum += m[(r * n + c) as usize]; }
    sum
}

fn row_sum_max(m: &[i64], n: i64) -> i64 {
    let mut best = row_sum(m, n, 0);
    for r in 1..n {
        let s = row_sum(m, n, r);
        if s > best { best = s; }
    }
    best
}

fn col_sum_max(m: &[i64], n: i64) -> i64 {
    let mut best = col_sum(m, n, 0);
    for c in 1..n {
        let s = col_sum(m, n, c);
        if s > best { best = s; }
    }
    best
}

fn is_symmetric(m: &[i64], n: i64) -> i64 {
    for i in 0..n {
        for j in 0..n {
            if m[(i * n + j) as usize] != m[(j * n + i) as usize] { return 0; }
        }
    }
    1
}

fn transpose_checksum(m: &[i64], n: i64) -> i64 {
    let mut sum = 0;
    for r in 0..n {
        for c in 0..n {
            let i = r * n + c;
            sum += i * m[(c * n + r) as usize];
        }
    }
    sum
}

fn matrix_add_checksum(a: &[i64], b: &[i64], n: i64) -> i64 {
    let mut sum = 0;
    for i in 0..n * n { sum += i * (a[i as usize] + b[i as usize]); }
    sum
}

fn scalar_mul_checksum(m: &[i64], n: i64, k: i64) -> i64 {
    let mut sum = 0;
    for i in 0..n * n { sum += i * (m[i as usize] * k); }
    sum
}

fn is_identity(m: &[i64], n: i64) -> i64 {
    for i in 0..n {
        for j in 0..n {
            if i == j {
                if m[(i * n + j) as usize] != 1 { return 0; }
            } else if m[(i * n + j) as usize] != 0 { return 0; }
        }
    }
    1
}

fn matrix_mul_trace(a: &[i64], b: &[i64], n: i64) -> i64 {
    let mut sum = 0;
    for i in 0..n {
        for k in 0..n {
            sum += a[(i * n + k) as usize] * b[(k * n + i) as usize];
        }
    }
    sum
}

fn main_diag_product(m: &[i64], n: i64) -> i64 {
    let mut product = 1;
    for i in 0..n { product *= m[(i * n + i) as usize]; }
    product
}

fn max_element(m: &[i64], n: i64) -> i64 {
    let mut best = m[0];
    for i in 1..n * n { if m[i as usize] > best { best = m[i as usize]; } }
    best
}

fn min_element(m: &[i64], n: i64) -> i64 {
    let mut best = m[0];
    for i in 1..n * n { if m[i as usize] < best { best = m[i as usize]; } }
    best
}

fn determinant_2x2(m: &[i64]) -> i64 {
    m[0] * m[3] - m[1] * m[2]
}

fn determinant_3x3(m: &[i64]) -> i64 {
    m[0] * (m[4] * m[8] - m[5] * m[7])
        - m[1] * (m[3] * m[8] - m[5] * m[6])
        + m[2] * (m[3] * m[7] - m[4] * m[6])
}

fn main() {
    let m = [1i64, 2, 3, 4, 5, 6, 7, 8, 9];
    let sym = [1i64, 2, 3, 2, 5, 6, 3, 6, 9];
    let id = [1i64, 0, 0, 0, 1, 0, 0, 0, 1];
    let b = [9i64, 8, 7, 6, 5, 4, 3, 2, 1];
    let d2 = [1i64, 2, 3, 4];
    println!("trace={}", trace(&m, 3));
    println!("diagonal_sum_main={}", diagonal_sum_main(&m, 3));
    println!("diagonal_sum_anti={}", diagonal_sum_anti(&m, 3));
    println!("row_sum={}", row_sum(&m, 3, 1));
    println!("col_sum={}", col_sum(&m, 3, 2));
    println!("row_sum_max={}", row_sum_max(&m, 3));
    println!("col_sum_max={}", col_sum_max(&m, 3));
    println!("is_symmetric={}", is_symmetric(&sym, 3));
    println!("transpose_checksum={}", transpose_checksum(&m, 3));
    println!("matrix_add_checksum={}", matrix_add_checksum(&m, &b, 3));
    println!("scalar_mul_checksum={}", scalar_mul_checksum(&m, 3, 2));
    println!("is_identity={}", is_identity(&id, 3));
    println!("matrix_mul_trace={}", matrix_mul_trace(&m, &b, 3));
    println!("main_diag_product={}", main_diag_product(&m, 3));
    println!("max_element={}", max_element(&m, 3));
    println!("min_element={}", min_element(&m, 3));
    println!("determinant_2x2={}", determinant_2x2(&d2));
    println!("determinant_3x3={}", determinant_3x3(&m));
}
