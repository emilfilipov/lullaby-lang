// Cross-language text-processing suite (C). Real-world string utilities mirroring
// ../lullaby.lby. Strings are `const char*`; length comes from strlen. Functions
// that build a string write into a caller-supplied buffer (idiomatic C). Boolean
// results are returned as int 1/0. See ../SPEC.md.

#include <assert.h>
#include <stdio.h>
#include <string.h>

typedef long long i64;

i64 word_count(const char *s) {
    i64 count = 0;
    int in_word = 0;
    for (const char *p = s; *p; p++) {
        if (*p == ' ' || *p == '\t' || *p == '\n') {
            in_word = 0;
        } else if (!in_word) {
            in_word = 1;
            count++;
        }
    }
    return count;
}

i64 char_count(const char *s) {
    return (i64)strlen(s);
}

i64 count_char(const char *s, const char *c) {
    i64 count = 0;
    for (const char *p = s; *p; p++)
        if (*p == c[0]) count++;
    return count;
}

i64 is_blank(const char *s) {
    for (const char *p = s; *p; p++)
        if (*p != ' ' && *p != '\t' && *p != '\n') return 0;
    return 1;
}

i64 starts_with_prefix(const char *s, const char *p) {
    return strncmp(s, p, strlen(p)) == 0 ? 1 : 0;
}

i64 ends_with_suffix(const char *s, const char *p) {
    size_t ls = strlen(s), lp = strlen(p);
    if (lp > ls) return 0;
    return strcmp(s + (ls - lp), p) == 0 ? 1 : 0;
}

i64 contains_sub(const char *s, const char *sub) {
    return strstr(s, sub) != NULL ? 1 : 0;
}

void to_upper_ascii(const char *s, char *out) {
    size_t i = 0;
    for (; s[i]; i++) {
        char c = s[i];
        out[i] = (c >= 'a' && c <= 'z') ? (char)(c - 32) : c;
    }
    out[i] = '\0';
}

void to_lower_ascii(const char *s, char *out) {
    size_t i = 0;
    for (; s[i]; i++) {
        char c = s[i];
        out[i] = (c >= 'A' && c <= 'Z') ? (char)(c + 32) : c;
    }
    out[i] = '\0';
}

void reverse_str(const char *s, char *out) {
    size_t n = strlen(s);
    for (size_t i = 0; i < n; i++)
        out[i] = s[n - 1 - i];
    out[n] = '\0';
}

void repeat_str(const char *s, i64 n, char *out) {
    size_t len = strlen(s), pos = 0;
    for (i64 i = 0; i < n; i++) {
        memcpy(out + pos, s, len);
        pos += len;
    }
    out[pos] = '\0';
}

void left_pad(const char *s, i64 width, const char *c, char *out) {
    size_t len = strlen(s), pos = 0;
    for (i64 pad = (i64)(width - (i64)len); pad > 0; pad--)
        out[pos++] = c[0];
    strcpy(out + pos, s);
}

void truncate_ellipsis(const char *s, i64 max, char *out) {
    size_t n = strlen(s);
    if ((i64)n <= max) {
        strcpy(out, s);
        return;
    }
    size_t keep = (size_t)(max - 3);
    memcpy(out, s, keep);
    strcpy(out + keep, "...");
}

i64 count_vowels(const char *s) {
    i64 count = 0;
    for (const char *p = s; *p; p++)
        if (*p == 'a' || *p == 'e' || *p == 'i' || *p == 'o' || *p == 'u') count++;
    return count;
}

void initials(const char *name, char *out) {
    size_t pos = 0;
    int in_word = 0;
    for (const char *p = name; *p; p++) {
        if (*p == ' ' || *p == '\t' || *p == '\n') {
            in_word = 0;
        } else if (!in_word) {
            in_word = 1;
            char c = *p;
            out[pos++] = (c >= 'a' && c <= 'z') ? (char)(c - 32) : c;
        }
    }
    out[pos] = '\0';
}

int main(void) {
    char buf[256];
    assert(word_count("the quick brown fox") == 4);
    assert(char_count("hello") == 5);
    assert(count_char("banana", "a") == 3);
    assert(is_blank("   ") == 1);
    assert(is_blank(" x ") == 0);
    assert(starts_with_prefix("lullaby", "lull") == 1);
    assert(ends_with_suffix("lullaby", "aby") == 1);
    assert(contains_sub("hello world", "o w") == 1);
    to_upper_ascii("Hello, World", buf);
    assert(strcmp(buf, "HELLO, WORLD") == 0);
    to_lower_ascii("Hello, World", buf);
    assert(strcmp(buf, "hello, world") == 0);
    reverse_str("lullaby", buf);
    assert(strcmp(buf, "yballul") == 0);
    repeat_str("ab", 3, buf);
    assert(strcmp(buf, "ababab") == 0);
    left_pad("42", 5, "0", buf);
    assert(strcmp(buf, "00042") == 0);
    truncate_ellipsis("hello world", 8, buf);
    assert(strcmp(buf, "hello...") == 0);
    assert(count_vowels("education") == 5);
    initials("grace brewster hopper", buf);
    assert(strcmp(buf, "GBH") == 0);
    printf("ok\n");
    return 0;
}
