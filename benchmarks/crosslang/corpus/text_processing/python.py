# Cross-language text-processing suite (Python). Real-world string utilities
# mirroring ../lullaby.lby. `s`/`c`/`sub`/`p` are str; boolean results are returned
# as int 1/0 for cross-language uniformity. ASCII-only case folding. See ../SPEC.md.


def word_count(s):
    return len(s.split())


def char_count(s):
    return len(s)


def count_char(s, c):
    return s.count(c[0])


def is_blank(s):
    return 1 if s.strip() == "" else 0


def starts_with_prefix(s, p):
    return 1 if s.startswith(p) else 0


def ends_with_suffix(s, p):
    return 1 if s.endswith(p) else 0


def contains_sub(s, sub):
    return 1 if sub in s else 0


def to_upper_ascii(s):
    return "".join(chr(ord(c) - 32) if "a" <= c <= "z" else c for c in s)


def to_lower_ascii(s):
    return "".join(chr(ord(c) + 32) if "A" <= c <= "Z" else c for c in s)


def reverse_str(s):
    return s[::-1]


def repeat_str(s, n):
    return s * n if n > 0 else ""


def left_pad(s, width, c):
    pad = width - len(s)
    return c[0] * pad + s if pad > 0 else s


def truncate_ellipsis(s, max):
    if len(s) <= max:
        return s
    return s[: max - 3] + "..."


def count_vowels(s):
    count = 0
    for c in s:
        if c in "aeiou":
            count += 1
    return count


def initials(name):
    return "".join(word[0].upper() for word in name.split())


if __name__ == "__main__":
    assert word_count("the quick brown fox") == 4
    assert char_count("hello") == 5
    assert count_char("banana", "a") == 3
    assert is_blank("   ") == 1
    assert is_blank(" x ") == 0
    assert starts_with_prefix("lullaby", "lull") == 1
    assert ends_with_suffix("lullaby", "aby") == 1
    assert contains_sub("hello world", "o w") == 1
    assert to_upper_ascii("Hello, World") == "HELLO, WORLD"
    assert to_lower_ascii("Hello, World") == "hello, world"
    assert reverse_str("lullaby") == "yballul"
    assert repeat_str("ab", 3) == "ababab"
    assert left_pad("42", 5, "0") == "00042"
    assert truncate_ellipsis("hello world", 8) == "hello..."
    assert count_vowels("education") == 5
    assert initials("grace brewster hopper") == "GBH"
    print("ok")
