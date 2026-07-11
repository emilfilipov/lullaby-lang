// Cross-language text-processing suite (C++). Real-world string utilities mirroring
// ../lullaby.lby, using std::string. `c`/`sub`/`p` are std::string; boolean results
// are returned as long long 1/0 for cross-language uniformity. See ../SPEC.md.

#include <cassert>
#include <cctype>
#include <iostream>
#include <string>

long long word_count(const std::string &s) {
    long long count = 0;
    bool in_word = false;
    for (char c : s) {
        if (std::isspace(static_cast<unsigned char>(c))) {
            in_word = false;
        } else if (!in_word) {
            in_word = true;
            count++;
        }
    }
    return count;
}

long long char_count(const std::string &s) {
    return static_cast<long long>(s.size());
}

long long count_char(const std::string &s, const std::string &c) {
    long long count = 0;
    for (char ch : s)
        if (ch == c[0]) count++;
    return count;
}

long long is_blank(const std::string &s) {
    for (char c : s)
        if (!std::isspace(static_cast<unsigned char>(c))) return 0;
    return 1;
}

long long starts_with_prefix(const std::string &s, const std::string &p) {
    return s.rfind(p, 0) == 0 ? 1 : 0;
}

long long ends_with_suffix(const std::string &s, const std::string &p) {
    if (p.size() > s.size()) return 0;
    return s.compare(s.size() - p.size(), p.size(), p) == 0 ? 1 : 0;
}

long long contains_sub(const std::string &s, const std::string &sub) {
    return s.find(sub) != std::string::npos ? 1 : 0;
}

std::string to_upper_ascii(const std::string &s) {
    std::string out;
    out.reserve(s.size());
    for (char c : s)
        out += (c >= 'a' && c <= 'z') ? static_cast<char>(c - 32) : c;
    return out;
}

std::string to_lower_ascii(const std::string &s) {
    std::string out;
    out.reserve(s.size());
    for (char c : s)
        out += (c >= 'A' && c <= 'Z') ? static_cast<char>(c + 32) : c;
    return out;
}

std::string reverse_str(const std::string &s) {
    return std::string(s.rbegin(), s.rend());
}

std::string repeat_str(const std::string &s, long long n) {
    std::string out;
    out.reserve(s.size() * (n > 0 ? n : 0));
    for (long long i = 0; i < n; i++)
        out += s;
    return out;
}

std::string left_pad(const std::string &s, long long width, const std::string &c) {
    long long pad = width - static_cast<long long>(s.size());
    if (pad <= 0) return s;
    return std::string(pad, c[0]) + s;
}

std::string truncate_ellipsis(const std::string &s, long long max) {
    if (static_cast<long long>(s.size()) <= max) return s;
    return s.substr(0, max - 3) + "...";
}

long long count_vowels(const std::string &s) {
    long long count = 0;
    for (char c : s)
        if (c == 'a' || c == 'e' || c == 'i' || c == 'o' || c == 'u') count++;
    return count;
}

std::string initials(const std::string &name) {
    std::string out;
    bool in_word = false;
    for (char c : name) {
        if (std::isspace(static_cast<unsigned char>(c))) {
            in_word = false;
        } else if (!in_word) {
            in_word = true;
            out += (c >= 'a' && c <= 'z') ? static_cast<char>(c - 32) : c;
        }
    }
    return out;
}

int main() {
    assert(word_count("the quick brown fox") == 4);
    assert(char_count("hello") == 5);
    assert(count_char("banana", "a") == 3);
    assert(is_blank("   ") == 1);
    assert(is_blank(" x ") == 0);
    assert(starts_with_prefix("lullaby", "lull") == 1);
    assert(ends_with_suffix("lullaby", "aby") == 1);
    assert(contains_sub("hello world", "o w") == 1);
    assert(to_upper_ascii("Hello, World") == "HELLO, WORLD");
    assert(to_lower_ascii("Hello, World") == "hello, world");
    assert(reverse_str("lullaby") == "yballul");
    assert(repeat_str("ab", 3) == "ababab");
    assert(left_pad("42", 5, "0") == "00042");
    assert(truncate_ellipsis("hello world", 8) == "hello...");
    assert(count_vowels("education") == 5);
    assert(initials("grace brewster hopper") == "GBH");
    std::cout << "ok" << std::endl;
    return 0;
}
