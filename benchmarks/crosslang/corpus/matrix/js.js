// Cross-language matrix suite (JavaScript). Dense matrix operations over a flat
// row-major array: an n*n matrix is passed as an array plus its order n, with
// element (r, c) stored at m[r * n + c].

function diagonal_sum_main(m, n) {
  let total = 0;
  for (let i = 0; i < n; i++) {
    total += m[i * n + i];
  }
  return total;
}

function trace(m, n) {
  return diagonal_sum_main(m, n);
}

function diagonal_sum_anti(m, n) {
  let total = 0;
  for (let i = 0; i < n; i++) {
    total += m[i * n + (n - 1 - i)];
  }
  return total;
}

function row_sum(m, n, r) {
  let total = 0;
  for (let c = 0; c < n; c++) {
    total += m[r * n + c];
  }
  return total;
}

function col_sum(m, n, c) {
  let total = 0;
  for (let r = 0; r < n; r++) {
    total += m[r * n + c];
  }
  return total;
}

function row_sum_max(m, n) {
  let best = row_sum(m, n, 0);
  for (let r = 1; r < n; r++) {
    const s = row_sum(m, n, r);
    if (s > best) {
      best = s;
    }
  }
  return best;
}

function col_sum_max(m, n) {
  let best = col_sum(m, n, 0);
  for (let c = 1; c < n; c++) {
    const s = col_sum(m, n, c);
    if (s > best) {
      best = s;
    }
  }
  return best;
}

function is_symmetric(m, n) {
  for (let i = 0; i < n; i++) {
    for (let j = 0; j < n; j++) {
      if (m[i * n + j] !== m[j * n + i]) {
        return 0;
      }
    }
  }
  return 1;
}

function transpose_checksum(m, n) {
  let total = 0;
  for (let r = 0; r < n; r++) {
    for (let c = 0; c < n; c++) {
      const i = r * n + c;
      total += i * m[c * n + r];
    }
  }
  return total;
}

function matrix_add_checksum(a, b, n) {
  let total = 0;
  for (let i = 0; i < n * n; i++) {
    total += i * (a[i] + b[i]);
  }
  return total;
}

function scalar_mul_checksum(m, n, k) {
  let total = 0;
  for (let i = 0; i < n * n; i++) {
    total += i * (m[i] * k);
  }
  return total;
}

function is_identity(m, n) {
  for (let i = 0; i < n; i++) {
    for (let j = 0; j < n; j++) {
      if (i === j) {
        if (m[i * n + j] !== 1) {
          return 0;
        }
      } else if (m[i * n + j] !== 0) {
        return 0;
      }
    }
  }
  return 1;
}

function matrix_mul_trace(a, b, n) {
  let total = 0;
  for (let i = 0; i < n; i++) {
    for (let k = 0; k < n; k++) {
      total += a[i * n + k] * b[k * n + i];
    }
  }
  return total;
}

function main_diag_product(m, n) {
  let product = 1;
  for (let i = 0; i < n; i++) {
    product *= m[i * n + i];
  }
  return product;
}

function max_element(m, n) {
  let best = m[0];
  for (let i = 1; i < n * n; i++) {
    if (m[i] > best) {
      best = m[i];
    }
  }
  return best;
}

function min_element(m, n) {
  let best = m[0];
  for (let i = 1; i < n * n; i++) {
    if (m[i] < best) {
      best = m[i];
    }
  }
  return best;
}

function determinant_2x2(m) {
  return m[0] * m[3] - m[1] * m[2];
}

function determinant_3x3(m) {
  return m[0] * (m[4] * m[8] - m[5] * m[7])
    - m[1] * (m[3] * m[8] - m[5] * m[6])
    + m[2] * (m[3] * m[7] - m[4] * m[6]);
}

function main() {
  const m = [1, 2, 3, 4, 5, 6, 7, 8, 9];
  const sym = [1, 2, 3, 2, 5, 6, 3, 6, 9];
  const ident = [1, 0, 0, 0, 1, 0, 0, 0, 1];
  const b = [9, 8, 7, 6, 5, 4, 3, 2, 1];
  const d2 = [1, 2, 3, 4];
  console.log("trace=" + trace(m, 3));
  console.log("diagonal_sum_main=" + diagonal_sum_main(m, 3));
  console.log("diagonal_sum_anti=" + diagonal_sum_anti(m, 3));
  console.log("row_sum=" + row_sum(m, 3, 1));
  console.log("col_sum=" + col_sum(m, 3, 2));
  console.log("row_sum_max=" + row_sum_max(m, 3));
  console.log("col_sum_max=" + col_sum_max(m, 3));
  console.log("is_symmetric=" + is_symmetric(sym, 3));
  console.log("transpose_checksum=" + transpose_checksum(m, 3));
  console.log("matrix_add_checksum=" + matrix_add_checksum(m, b, 3));
  console.log("scalar_mul_checksum=" + scalar_mul_checksum(m, 3, 2));
  console.log("is_identity=" + is_identity(ident, 3));
  console.log("matrix_mul_trace=" + matrix_mul_trace(m, b, 3));
  console.log("main_diag_product=" + main_diag_product(m, 3));
  console.log("max_element=" + max_element(m, 3));
  console.log("min_element=" + min_element(m, 3));
  console.log("determinant_2x2=" + determinant_2x2(d2));
  console.log("determinant_3x3=" + determinant_3x3(m));
}

main();
