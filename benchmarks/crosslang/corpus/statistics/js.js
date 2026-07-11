// Cross-language statistics suite (JavaScript). Integer statistics over an array and
// a length: sums, spread, order statistics, and a Kadane max-subarray scan.

function total(a, n) {
  let s = 0;
  for (let i = 0; i < n; i++) s += a[i];
  return s;
}

function mean_floor(a, n) {
  return Math.trunc(total(a, n) / n);
}

function variance_scaled(a, n) {
  const mean = mean_floor(a, n);
  let s = 0;
  for (let i = 0; i < n; i++) {
    const d = a[i] - mean;
    s += d * d;
  }
  return s;
}

function min_val(a, n) {
  let m = a[0];
  for (let i = 1; i < n; i++) if (a[i] < m) m = a[i];
  return m;
}

function max_val(a, n) {
  let m = a[0];
  for (let i = 1; i < n; i++) if (a[i] > m) m = a[i];
  return m;
}

function range_span(a, n) {
  return max_val(a, n) - min_val(a, n);
}

function median_sorted(a, n) {
  return a[Math.trunc((n - 1) / 2)];
}

function mode_count_max(a, n) {
  if (n === 0) return 0;
  let best = 1;
  let run = 1;
  for (let i = 1; i < n; i++) {
    run = a[i] === a[i - 1] ? run + 1 : 1;
    if (run > best) best = run;
  }
  return best;
}

function count_above_mean(a, n) {
  const mean = mean_floor(a, n);
  let count = 0;
  for (let i = 0; i < n; i++) if (a[i] > mean) count += 1;
  return count;
}

function sum_abs_dev(a, n) {
  const mean = mean_floor(a, n);
  let s = 0;
  for (let i = 0; i < n; i++) s += Math.abs(a[i] - mean);
  return s;
}

function product_mod(a, n, m) {
  let prod = 1;
  for (let i = 0; i < n; i++) prod = (prod * a[i]) % m;
  return prod;
}

function running_max_sum(a, n) {
  let best = a[0];
  let cur = a[0];
  for (let i = 1; i < n; i++) {
    cur = cur + a[i] > a[i] ? cur + a[i] : a[i];
    if (cur > best) best = cur;
  }
  return best;
}

function zscore_sign_count(a, n) {
  const mean = mean_floor(a, n);
  let count = 0;
  for (let i = 0; i < n; i++) if (a[i] > mean) count += 1;
  return count;
}

function weighted_sum(a, w, n) {
  let s = 0;
  for (let i = 0; i < n; i++) s += a[i] * w[i];
  return s;
}

function cumulative_max_last(a, n) {
  let m = a[0];
  for (let i = 1; i < n; i++) if (a[i] > m) m = a[i];
  return m;
}

function main() {
  const d = [3, -7, 4, 8, -2, 5];
  const s = [2, 2, 2, 5, 7, 8];
  const w = [1, 2, 1, 3, 1, 2];
  console.log("total=" + total(d, 6));
  console.log("mean_floor=" + mean_floor(d, 6));
  console.log("variance_scaled=" + variance_scaled(d, 6));
  console.log("min_val=" + min_val(d, 6));
  console.log("max_val=" + max_val(d, 6));
  console.log("range_span=" + range_span(d, 6));
  console.log("median_sorted=" + median_sorted(s, 6));
  console.log("mode_count_max=" + mode_count_max(s, 6));
  console.log("count_above_mean=" + count_above_mean(d, 6));
  console.log("sum_abs_dev=" + sum_abs_dev(d, 6));
  console.log("product_mod=" + product_mod(s, 6, 1000));
  console.log("running_max_sum=" + running_max_sum(d, 6));
  console.log("zscore_sign_count=" + zscore_sign_count(d, 6));
  console.log("weighted_sum=" + weighted_sum(d, w, 6));
  console.log("cumulative_max_last=" + cumulative_max_last(d, 6));
}

main();
