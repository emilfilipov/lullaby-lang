// Cross-language collections suite (JavaScript). Array-as-collection algorithms:
// frequency, grouping, and set-like operations over an array and a length. No
// Map or Set is used: everything is counting and scanning, relying on sorted
// inputs where noted.

function count_frequency_of(a, n, x) {
  let count = 0;
  for (let i = 0; i < n; i++) if (a[i] === x) count++;
  return count;
}

function max_frequency(a, n) {
  if (n === 0) return 0;
  let best = 1, run = 1;
  for (let i = 1; i < n; i++) {
    run = a[i] === a[i - 1] ? run + 1 : 1;
    if (run > best) best = run;
  }
  return best;
}

function first_duplicate_value(a, n) {
  for (let i = 1; i < n; i++) if (a[i] === a[i - 1]) return a[i];
  return -1;
}

function has_pair_sum(a, n, target) {
  let lo = 0, hi = n - 1;
  while (lo < hi) {
    const s = a[lo] + a[hi];
    if (s === target) return 1;
    else if (s < target) lo++;
    else hi--;
  }
  return 0;
}

function count_distinct_sorted(a, n) {
  if (n === 0) return 0;
  let count = 1;
  for (let i = 1; i < n; i++) if (a[i] !== a[i - 1]) count++;
  return count;
}

function most_common_sorted(a, n) {
  let best_val = a[0], best = 1, run = 1;
  for (let i = 1; i < n; i++) {
    run = a[i] === a[i - 1] ? run + 1 : 1;
    if (run > best) { best = run; best_val = a[i]; }
  }
  return best_val;
}

function count_even(a, n) {
  let count = 0;
  for (let i = 0; i < n; i++) if (a[i] % 2 === 0) count++;
  return count;
}

function count_odd(a, n) {
  let count = 0;
  for (let i = 0; i < n; i++) if (a[i] % 2 !== 0) count++;
  return count;
}

function partition_point(a, n) {
  for (let i = 0; i < n; i++) if (a[i] >= 0) return i;
  return n;
}

function count_in_range(a, n, lo, hi) {
  let count = 0;
  for (let i = 0; i < n; i++) if (a[i] >= lo && a[i] <= hi) count++;
  return count;
}

function running_total_last(a, n) {
  let total = 0;
  for (let i = 0; i < n; i++) total += a[i];
  return total;
}

function zip_sum(a, b, n) {
  let sum = 0;
  for (let i = 0; i < n; i++) sum += a[i] + b[i];
  return sum;
}

function intersect_count_sorted(a, la, b, lb) {
  let i = 0, j = 0, count = 0;
  while (i < la && j < lb) {
    if (a[i] === b[j]) { count++; i++; j++; }
    else if (a[i] < b[j]) i++;
    else j++;
  }
  return count;
}

function union_count_sorted(a, la, b, lb) {
  let i = 0, j = 0, count = 0;
  while (i < la && j < lb) {
    if (a[i] === b[j]) { count++; i++; j++; }
    else if (a[i] < b[j]) { count++; i++; }
    else { count++; j++; }
  }
  while (i < la) { count++; i++; }
  while (j < lb) { count++; j++; }
  return count;
}

function is_subset_sorted(a, la, b, lb) {
  let i = 0, j = 0;
  while (i < la && j < lb) {
    if (a[i] === b[j]) { i++; j++; }
    else if (a[i] > b[j]) j++;
    else return 0;
  }
  return i < la ? 0 : 1;
}

function rotate_left_checksum(a, n, k) {
  const shift = k % n;
  let sum = 0;
  for (let i = 0; i < n; i++) {
    let idx = i + shift;
    if (idx >= n) idx -= n;
    sum += i * a[idx];
  }
  return sum;
}

function dedup_sorted_checksum(a, n) {
  if (n === 0) return 0;
  let sum = 0, pos = 0, prev = a[0];
  for (let i = 1; i < n; i++) {
    if (a[i] !== prev) { pos++; sum += pos * a[i]; prev = a[i]; }
  }
  return sum;
}

function chunk_sum_max(a, n, k) {
  let window = 0;
  for (let i = 0; i < k; i++) window += a[i];
  let best = window;
  for (let i = k; i < n; i++) {
    window += a[i] - a[i - k];
    if (window > best) best = window;
  }
  return best;
}

function main() {
  const a = [-5, -2, -2, 0, 1, 1, 1, 4, 7, 7];
  const b = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
  const c = [1, 3, 5, 7, 9];
  const d = [2, 3, 5, 8, 9];
  const e = [3, 5, 9];
  console.log("count_frequency_of=" + count_frequency_of(a, 10, 1));
  console.log("max_frequency=" + max_frequency(a, 10));
  console.log("first_duplicate_value=" + first_duplicate_value(a, 10));
  console.log("has_pair_sum=" + has_pair_sum(a, 10, 2));
  console.log("count_distinct_sorted=" + count_distinct_sorted(a, 10));
  console.log("most_common_sorted=" + most_common_sorted(a, 10));
  console.log("count_even=" + count_even(a, 10));
  console.log("count_odd=" + count_odd(a, 10));
  console.log("partition_point=" + partition_point(a, 10));
  console.log("count_in_range=" + count_in_range(a, 10, -2, 1));
  console.log("running_total_last=" + running_total_last(a, 10));
  console.log("zip_sum=" + zip_sum(a, b, 10));
  console.log("intersect_count_sorted=" + intersect_count_sorted(c, 5, d, 5));
  console.log("union_count_sorted=" + union_count_sorted(c, 5, d, 5));
  console.log("is_subset_sorted=" + is_subset_sorted(e, 3, d, 5));
  console.log("rotate_left_checksum=" + rotate_left_checksum(b, 10, 3));
  console.log("dedup_sorted_checksum=" + dedup_sorted_checksum(a, 10));
  console.log("chunk_sum_max=" + chunk_sum_max(b, 10, 3));
}

main();
