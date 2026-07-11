// Cross-language text-processing suite (Rust). Real-world string utilities mirroring
// ../lullaby.lby. Inputs are `&str`; `c`/`sub`/`p` are `&str`; boolean results are
// returned as i64 1/0 for cross-language uniformity. ASCII-only case folding. See ../SPEC.md.
#![allow(dead_code)]

fn word_count(s: &str) -> i64 {
    s.split_whitespace().count() as i64
}

fn char_count(s: &str) -> i64 {
    s.chars().count() as i64
}

fn count_char(s: &str, c: &str) -> i64 {
    let target = c.chars().next().unwrap();
    s.chars().filter(|&ch| ch == target).count() as i64
}

fn is_blank(s: &str) -> i64 {
    if s.chars().all(|c| c.is_whitespace()) { 1 } else { 0 }
}

fn starts_with_prefix(s: &str, p: &str) -> i64 {
    if s.starts_with(p) { 1 } else { 0 }
}

fn ends_with_suffix(s: &str, p: &str) -> i64 {
    if s.ends_with(p) { 1 } else { 0 }
}

fn contains_sub(s: &str, sub: &str) -> i64 {
    if s.contains(sub) { 1 } else { 0 }
}

fn to_upper_ascii(s: &str) -> String {
    s.to_ascii_uppercase()
}

fn to_lower_ascii(s: &str) -> String {
    s.to_ascii_lowercase()
}

fn reverse_str(s: &str) -> String {
    s.chars().rev().collect()
}

fn repeat_str(s: &str, n: i64) -> String {
    if n <= 0 {
        String::new()
    } else {
        s.repeat(n as usize)
    }
}

fn left_pad(s: &str, width: i64, c: &str) -> String {
    let pad = width - s.chars().count() as i64;
    if pad <= 0 {
        s.to_string()
    } else {
        c.repeat(pad as usize) + s
    }
}

fn truncate_ellipsis(s: &str, max: i64) -> String {
    if s.chars().count() as i64 <= max {
        s.to_string()
    } else {
        let keep = (max - 3) as usize;
        let head: String = s.chars().take(keep).collect();
        head + "..."
    }
}

fn count_vowels(s: &str) -> i64 {
    s.chars().filter(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')).count() as i64
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

fn main() {
    assert_eq!(word_count("the quick brown fox"), 4);
    assert_eq!(char_count("hello"), 5);
    assert_eq!(count_char("banana", "a"), 3);
    assert_eq!(is_blank("   "), 1);
    assert_eq!(is_blank(" x "), 0);
    assert_eq!(starts_with_prefix("lullaby", "lull"), 1);
    assert_eq!(ends_with_suffix("lullaby", "aby"), 1);
    assert_eq!(contains_sub("hello world", "o w"), 1);
    assert_eq!(to_upper_ascii("Hello, World"), "HELLO, WORLD");
    assert_eq!(to_lower_ascii("Hello, World"), "hello, world");
    assert_eq!(reverse_str("lullaby"), "yballul");
    assert_eq!(repeat_str("ab", 3), "ababab");
    assert_eq!(left_pad("42", 5, "0"), "00042");
    assert_eq!(truncate_ellipsis("hello world", 8), "hello...");
    assert_eq!(count_vowels("education"), 5);
    assert_eq!(initials("grace brewster hopper"), "GBH");
    println!("ok");
}
