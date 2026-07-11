// Cross-language sorting suite (JavaScript, Node). Classic sort algorithms and
// order statistics over an array and a length. Each function returns a scalar
// (a checksum sum(i*sorted[i]), a count, or an index). Each function sorts a
// local copy so the caller's array is left untouched.

function checksum(a, n) {
  let sum = 0;
  for (let i = 0; i < n; i++) sum += i * a[i];
  return sum;
}

function insertion_sort_checksum(a, n) {
  const buf = a.slice(0, n);
  for (let i = 1; i < n; i++) {
    const key = buf[i];
    let j = i - 1;
    while (j >= 0 && buf[j] > key) {
      buf[j + 1] = buf[j];
      j--;
    }
    buf[j + 1] = key;
  }
  return checksum(buf, n);
}

function selection_sort_checksum(a, n) {
  const buf = a.slice(0, n);
  for (let i = 0; i < n - 1; i++) {
    let mi = i;
    for (let j = i + 1; j < n; j++) if (buf[j] < buf[mi]) mi = j;
    [buf[i], buf[mi]] = [buf[mi], buf[i]];
  }
  return checksum(buf, n);
}

function bubble_sort_swaps(a, n) {
  const buf = a.slice(0, n);
  let swaps = 0;
  for (let i = 0; i < n; i++)
    for (let j = 0; j < n - 1 - i; j++)
      if (buf[j] > buf[j + 1]) {
        [buf[j], buf[j + 1]] = [buf[j + 1], buf[j]];
        swaps++;
      }
  return swaps;
}

function gnome_sort_checksum(a, n) {
  const buf = a.slice(0, n);
  let i = 0;
  while (i < n) {
    if (i === 0 || buf[i] >= buf[i - 1]) {
      i++;
    } else {
      [buf[i], buf[i - 1]] = [buf[i - 1], buf[i]];
      i--;
    }
  }
  return checksum(buf, n);
}

function cocktail_sort_checksum(a, n) {
  const buf = a.slice(0, n);
  let lo = 0, hi = n - 1, swapped = true;
  while (swapped) {
    swapped = false;
    for (let i = lo; i < hi; i++)
      if (buf[i] > buf[i + 1]) {
        [buf[i], buf[i + 1]] = [buf[i + 1], buf[i]];
        swapped = true;
      }
    if (!swapped) break;
    hi--;
    swapped = false;
    for (let i = hi - 1; i >= lo; i--)
      if (buf[i] > buf[i + 1]) {
        [buf[i], buf[i + 1]] = [buf[i + 1], buf[i]];
        swapped = true;
      }
    lo++;
  }
  return checksum(buf, n);
}

function comb_sort_checksum(a, n) {
  const buf = a.slice(0, n);
  let gap = n, swapped = true;
  while (gap > 1 || swapped) {
    gap = Math.floor((gap * 10) / 13);
    if (gap < 1) gap = 1;
    swapped = false;
    for (let i = 0; i < n - gap; i++)
      if (buf[i] > buf[i + gap]) {
        [buf[i], buf[i + gap]] = [buf[i + gap], buf[i]];
        swapped = true;
      }
  }
  return checksum(buf, n);
}

function count_inversions(a, n) {
  let count = 0;
  for (let i = 0; i < n; i++)
    for (let j = i + 1; j < n; j++) if (a[i] > a[j]) count++;
  return count;
}

function is_sorted_desc(a, n) {
  for (let i = 1; i < n; i++) if (a[i] > a[i - 1]) return 0;
  return 1;
}

function merge_two_sorted_checksum(a, la, b, lb) {
  let i = 0, j = 0, k = 0, sum = 0;
  while (i < la && j < lb) {
    if (a[i] <= b[j]) { sum += k * a[i]; i++; }
    else { sum += k * b[j]; j++; }
    k++;
  }
  while (i < la) { sum += k * a[i]; i++; k++; }
  while (j < lb) { sum += k * b[j]; j++; k++; }
  return sum;
}

function partition_lomuto_index(a, n) {
  const buf = a.slice(0, n);
  const pivot = buf[n - 1];
  let i = 0;
  for (let j = 0; j < n - 1; j++)
    if (buf[j] < pivot) {
      [buf[i], buf[j]] = [buf[j], buf[i]];
      i++;
    }
  [buf[i], buf[n - 1]] = [buf[n - 1], buf[i]];
  return i;
}

function kth_smallest(a, n, k) {
  const buf = a.slice(0, n);
  for (let i = 0; i < k; i++) {
    let mi = i;
    for (let j = i + 1; j < n; j++) if (buf[j] < buf[mi]) mi = j;
    [buf[i], buf[mi]] = [buf[mi], buf[i]];
  }
  return buf[k - 1];
}

function count_sorted_runs(a, n) {
  if (n === 0) return 0;
  let runs = 1;
  for (let i = 1; i < n; i++) if (a[i] < a[i - 1]) runs++;
  return runs;
}

function min_swaps_selection(a, n) {
  const buf = a.slice(0, n);
  let swaps = 0;
  for (let i = 0; i < n - 1; i++) {
    let mi = i;
    for (let j = i + 1; j < n; j++) if (buf[j] < buf[mi]) mi = j;
    if (mi !== i) {
      [buf[i], buf[mi]] = [buf[mi], buf[i]];
      swaps++;
    }
  }
  return swaps;
}

function sorted_median(a, n) {
  const s = a.slice(0, n).sort((x, y) => x - y);
  return s[Math.floor(n / 2)];
}

function sort_evens_first_checksum(a, n) {
  const s = a.slice(0, n).sort((x, y) => x - y);
  let sum = 0, k = 0;
  for (let i = 0; i < n; i++)
    if (s[i] % 2 === 0) { sum += k * s[i]; k++; }
  for (let i = 0; i < n; i++)
    if (s[i] % 2 !== 0) { sum += k * s[i]; k++; }
  return sum;
}

function reverse_checksum(a, n) {
  let sum = 0;
  for (let i = 0; i < n; i++) sum += i * a[n - 1 - i];
  return sum;
}

function max_gap_sorted(a, n) {
  const s = a.slice(0, n).sort((x, y) => x - y);
  let g = 0;
  for (let i = 1; i < n; i++) {
    const d = s[i] - s[i - 1];
    if (d > g) g = d;
  }
  return g;
}

function second_smallest(a, n) {
  let first = a[0], second = a[1];
  if (second < first) [first, second] = [second, first];
  for (let i = 2; i < n; i++) {
    if (a[i] < first) { second = first; first = a[i]; }
    else if (a[i] < second) second = a[i];
  }
  return second;
}

function main() {
  const t = [5, 3, 8, 1, 9, 2];
  const p = [1, 4, 6, 8];
  const q = [2, 3, 5, 7, 9];
  console.log("insertion_sort_checksum=" + insertion_sort_checksum(t, 6));
  console.log("selection_sort_checksum=" + selection_sort_checksum(t, 6));
  console.log("bubble_sort_swaps=" + bubble_sort_swaps(t, 6));
  console.log("gnome_sort_checksum=" + gnome_sort_checksum(t, 6));
  console.log("cocktail_sort_checksum=" + cocktail_sort_checksum(t, 6));
  console.log("comb_sort_checksum=" + comb_sort_checksum(t, 6));
  console.log("count_inversions=" + count_inversions(t, 6));
  console.log("is_sorted_desc=" + is_sorted_desc(t, 6));
  console.log("merge_two_sorted_checksum=" + merge_two_sorted_checksum(p, 4, q, 5));
  console.log("partition_lomuto_index=" + partition_lomuto_index(t, 6));
  console.log("kth_smallest=" + kth_smallest(t, 6, 3));
  console.log("count_sorted_runs=" + count_sorted_runs(t, 6));
  console.log("min_swaps_selection=" + min_swaps_selection(t, 6));
  console.log("sorted_median=" + sorted_median(t, 6));
  console.log("sort_evens_first_checksum=" + sort_evens_first_checksum(t, 6));
  console.log("reverse_checksum=" + reverse_checksum(t, 6));
  console.log("max_gap_sorted=" + max_gap_sorted(t, 6));
  console.log("second_smallest=" + second_smallest(t, 6));
}

main();
