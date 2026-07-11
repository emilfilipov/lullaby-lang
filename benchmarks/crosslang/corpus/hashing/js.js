// Cross-language hashing suite (JavaScript). Deterministic integer hashes over
// an integer byte array (values 0..255) plus its length. Arithmetic only — no
// bitwise operators, to stay algorithm-identical to Lullaby. Numbers are IEEE
// doubles, so moduli and products are kept below 2^53 to stay exact; integer
// division uses Math.trunc.

function djb2_hash(a, n) {
    const m = 1000000007;
    let h = 5381;
    for (let i = 0; i < n; i++) h = (h * 33 + a[i]) % m;
    return h;
}

function fnv1a_arithmetic(a, n) {
    const m = 100000007;
    const prime = 16777619;
    let h = 2166136261 % m;
    for (let i = 0; i < n; i++) h = ((h + a[i]) * prime) % m;
    return h;
}

function sum_hash(a, n) {
    let total = 0;
    for (let i = 0; i < n; i++) total += a[i];
    return total;
}

function poly_hash(a, n, base, modulus) {
    let h = 0;
    for (let i = 0; i < n; i++) h = (h * base + a[i]) % modulus;
    return h;
}

function rolling_hash(a, n, modulus) {
    let h = 0;
    for (let i = 0; i < n; i++) h = (h * 256 + a[i]) % modulus;
    return h;
}

function sdbm_hash(a, n) {
    const m = 1000000007;
    let h = 0;
    for (let i = 0; i < n; i++) h = (a[i] + h * 65599) % m;
    return h;
}

function mod_sum_hash(a, n, m) {
    let h = 0;
    for (let i = 0; i < n; i++) h = (h + a[i]) % m;
    return h;
}

function weighted_pos_hash(a, n) {
    const m = 1000000007;
    let h = 0;
    for (let i = 0; i < n; i++) h = (h + (i + 1) * a[i]) % m;
    return h;
}

function xor_free_checksum(a, n) {
    let total = 0;
    for (let i = 0; i < n; i++) total += a[i];
    return total % 256;
}

function product_hash(a, n, m) {
    let h = 1;
    for (let i = 0; i < n; i++) h = (h * (a[i] + 1)) % m;
    return h;
}

function rotate_hash(a, n) {
    const m = 1073741824;
    const half = 536870912;
    let h = 0;
    for (let i = 0; i < n; i++) {
        const carry = Math.trunc(h / half);
        h = (h * 2) % m + carry;
        h = (h + a[i]) % m;
    }
    return h;
}

function pearson_like(a, n) {
    const t = [7, 3, 11, 15, 0, 9, 5, 13, 1, 14, 6, 10, 2, 12, 4, 8];
    let h = 0;
    for (let i = 0; i < n; i++) h = t[(h + a[i]) % 16];
    return h;
}

function length_hash(a, n) {
    const m = 1000000007;
    if (n <= 0) return 0;
    return (n * 2654435761 + a[0]) % m;
}

function first_last_hash(a, n) {
    if (n <= 0) return 0;
    return a[0] * 257 + a[n - 1];
}

function midpoint_hash(a, n) {
    if (n <= 0) return 0;
    return a[Math.trunc(n / 2)] * 33 + n;
}

function digit_hash(x) {
    if (x < 0) x = -x;
    if (x === 0) return 0;
    const m = 1000000007;
    let h = 7;
    while (x > 0) {
        h = (h * 31 + (x % 10)) % m;
        x = Math.trunc(x / 10);
    }
    return h;
}

function pair_hash(a, b) {
    const s = a + b;
    return Math.trunc(s * (s + 1) / 2) + b;
}

function main() {
    const data = [72, 101, 108, 108, 111];
    console.log("djb2_hash=" + djb2_hash(data, 5));
    console.log("fnv1a_arithmetic=" + fnv1a_arithmetic(data, 5));
    console.log("sum_hash=" + sum_hash(data, 5));
    console.log("poly_hash=" + poly_hash(data, 5, 31, 1000000007));
    console.log("rolling_hash=" + rolling_hash(data, 5, 1000000007));
    console.log("sdbm_hash=" + sdbm_hash(data, 5));
    console.log("mod_sum_hash=" + mod_sum_hash(data, 5, 97));
    console.log("weighted_pos_hash=" + weighted_pos_hash(data, 5));
    console.log("xor_free_checksum=" + xor_free_checksum(data, 5));
    console.log("product_hash=" + product_hash(data, 5, 1000000007));
    console.log("rotate_hash=" + rotate_hash(data, 5));
    console.log("pearson_like=" + pearson_like(data, 5));
    console.log("length_hash=" + length_hash(data, 5));
    console.log("first_last_hash=" + first_last_hash(data, 5));
    console.log("midpoint_hash=" + midpoint_hash(data, 5));
    console.log("digit_hash=" + digit_hash(1234));
    console.log("pair_hash=" + pair_hash(17, 42));
}

main();
