// Cross-language text-processing suite (JavaScript). Real-world string utilities
// mirroring ../lullaby.lby. `s`/`c`/`sub`/`p` are strings; boolean results are returned
// as int 1/0 for cross-language uniformity. ASCII-only case folding. See ../SPEC.md.

const assert = require("assert");

function word_count(s) {
  const t = s.trim();
  return t === "" ? 0 : t.split(/\s+/).length;
}

function char_count(s) {
  return s.length;
}

function count_char(s, c) {
  const target = c[0];
  let count = 0;
  for (let i = 0; i < s.length; i++) if (s[i] === target) count += 1;
  return count;
}

function is_blank(s) {
  return s.trim() === "" ? 1 : 0;
}

function starts_with_prefix(s, p) {
  return s.startsWith(p) ? 1 : 0;
}

function ends_with_suffix(s, p) {
  return s.endsWith(p) ? 1 : 0;
}

function contains_sub(s, sub) {
  return s.includes(sub) ? 1 : 0;
}

function to_upper_ascii(s) {
  let out = "";
  for (let i = 0; i < s.length; i++) {
    const code = s.charCodeAt(i);
    out += code >= 97 && code <= 122 ? String.fromCharCode(code - 32) : s[i];
  }
  return out;
}

function to_lower_ascii(s) {
  let out = "";
  for (let i = 0; i < s.length; i++) {
    const code = s.charCodeAt(i);
    out += code >= 65 && code <= 90 ? String.fromCharCode(code + 32) : s[i];
  }
  return out;
}

function reverse_str(s) {
  let out = "";
  for (let i = s.length - 1; i >= 0; i--) out += s[i];
  return out;
}

function repeat_str(s, n) {
  return n > 0 ? s.repeat(n) : "";
}

function left_pad(s, width, c) {
  const pad = width - s.length;
  return pad > 0 ? c[0].repeat(pad) + s : s;
}

function truncate_ellipsis(s, max) {
  if (s.length <= max) return s;
  return s.slice(0, max - 3) + "...";
}

function count_vowels(s) {
  let count = 0;
  for (let i = 0; i < s.length; i++) {
    const c = s[i];
    if (c === "a" || c === "e" || c === "i" || c === "o" || c === "u") count += 1;
  }
  return count;
}

function initials(name) {
  const t = name.trim();
  if (t === "") return "";
  let out = "";
  for (const word of t.split(/\s+/)) out += word[0].toUpperCase();
  return out;
}

function main() {
  assert.strictEqual(word_count("the quick brown fox"), 4);
  assert.strictEqual(char_count("hello"), 5);
  assert.strictEqual(count_char("banana", "a"), 3);
  assert.strictEqual(is_blank("   "), 1);
  assert.strictEqual(is_blank(" x "), 0);
  assert.strictEqual(starts_with_prefix("lullaby", "lull"), 1);
  assert.strictEqual(ends_with_suffix("lullaby", "aby"), 1);
  assert.strictEqual(contains_sub("hello world", "o w"), 1);
  assert.strictEqual(to_upper_ascii("Hello, World"), "HELLO, WORLD");
  assert.strictEqual(to_lower_ascii("Hello, World"), "hello, world");
  assert.strictEqual(reverse_str("lullaby"), "yballul");
  assert.strictEqual(repeat_str("ab", 3), "ababab");
  assert.strictEqual(left_pad("42", 5, "0"), "00042");
  assert.strictEqual(truncate_ellipsis("hello world", 8), "hello...");
  assert.strictEqual(count_vowels("education"), 5);
  assert.strictEqual(initials("grace brewster hopper"), "GBH");
  console.log("ok");
}

main();
