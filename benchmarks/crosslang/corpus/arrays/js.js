// Cross-language arrays suite (JavaScript). Real-world array/statistics
// operations over a JS array and a length. bubble_sort_checksum sorts a local
// copy so the caller's array is left untouched.

function sum_array(a, n) {
  let total = 0;
  for (let i = 0; i < n; i++) {
    total += a[i];
  }
  return total;
}

function max_array(a, n) {
  let m = a[0];
  for (let i = 1; i < n; i++) {
    if (a[i] > m) {
      m = a[i];
    }
  }
  return m;
}

function min_array(a, n) {
  let m = a[0];
  for (let i = 1; i < n; i++) {
    if (a[i] < m) {
      m = a[i];
    }
  }
  return m;
}

function mean_floor(a, n) {
  return Math.trunc(sum_array(a, n) / n);
}

function count_positive(a, n) {
  let count = 0;
  for (let i = 0; i < n; i++) {
    if (a[i] > 0) {
      count++;
    }
  }
  return count;
}

function count_equal(a, n, x) {
  let count = 0;
  for (let i = 0; i < n; i++) {
    if (a[i] === x) {
      count++;
    }
  }
  return count;
}

function index_of(a, n, x) {
  for (let i = 0; i < n; i++) {
    if (a[i] === x) {
      return i;
    }
  }
  return -1;
}

function binary_search(a, n, x) {
  let lo = 0;
  let hi = n - 1;
  while (lo <= hi) {
    const mid = Math.trunc((lo + hi) / 2);
    if (a[mid] === x) {
      return mid;
    } else if (a[mid] < x) {
      lo = mid + 1;
    } else {
      hi = mid - 1;
    }
  }
  return -1;
}

function is_sorted_asc(a, n) {
  for (let i = 1; i < n; i++) {
    if (a[i] < a[i - 1]) {
      return 0;
    }
  }
  return 1;
}

function range_span(a, n) {
  return max_array(a, n) - min_array(a, n);
}

function dot_product(a, b, n) {
  let total = 0;
  for (let i = 0; i < n; i++) {
    total += a[i] * b[i];
  }
  return total;
}

function count_distinct_sorted(a, n) {
  if (n === 0) {
    return 0;
  }
  let count = 1;
  for (let i = 1; i < n; i++) {
    if (a[i] !== a[i - 1]) {
      count++;
    }
  }
  return count;
}

function second_largest(a, n) {
  let first = a[0];
  let second = a[1];
  if (second > first) {
    const tmp = first;
    first = second;
    second = tmp;
  }
  for (let i = 2; i < n; i++) {
    if (a[i] > first) {
      second = first;
      first = a[i];
    } else if (a[i] > second) {
      second = a[i];
    }
  }
  return second;
}

function prefix_sum_last(a, n) {
  let prefix = 0;
  for (let i = 0; i < n; i++) {
    prefix += a[i];
  }
  return prefix;
}

function bubble_sort_checksum(a, n) {
  const buf = a.slice(0, n);
  for (let i = 0; i < n; i++) {
    for (let j = 0; j < n - 1 - i; j++) {
      if (buf[j] > buf[j + 1]) {
        const tmp = buf[j];
        buf[j] = buf[j + 1];
        buf[j + 1] = tmp;
      }
    }
  }
  let sum = 0;
  for (let i = 0; i < n; i++) {
    sum += i * buf[i];
  }
  return sum;
}

function main() {
  const t = [5, 3, 8, 1, 9, 2];
  const s = [1, 2, 2, 3, 5, 8];
  console.log("sum_array=" + sum_array(t, 6));
  console.log("max_array=" + max_array(t, 6));
  console.log("min_array=" + min_array(t, 6));
  console.log("mean_floor=" + mean_floor(t, 6));
  console.log("count_positive=" + count_positive(t, 6));
  console.log("count_equal=" + count_equal(t, 6, 8));
  console.log("index_of=" + index_of(t, 6, 1));
  console.log("binary_search=" + binary_search(s, 6, 5));
  console.log("is_sorted_asc=" + is_sorted_asc(s, 6));
  console.log("range_span=" + range_span(t, 6));
  console.log("dot_product=" + dot_product(t, s, 6));
  console.log("count_distinct_sorted=" + count_distinct_sorted(s, 6));
  console.log("second_largest=" + second_largest(t, 6));
  console.log("prefix_sum_last=" + prefix_sum_last(t, 6));
  console.log("bubble_sort_checksum=" + bubble_sort_checksum(t, 6));
}

main();
