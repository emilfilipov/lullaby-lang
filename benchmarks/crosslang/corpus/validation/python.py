# Cross-language validation suite (Python). Same real-world validators as
# ../validation/lullaby.lby, written with plain loops (no regex / str.isdigit /
# calendar module) so the logic is comparable across languages. See ../../SPEC.md.


def luhn_check(digits, n):
    total = 0
    for i in range(n):
        d = digits[i]
        pos = n - 1 - i
        if pos % 2 == 1:
            d *= 2
            if d > 9:
                d -= 9
        total += d
    return 1 if total % 10 == 0 else 0


def is_valid_isbn10(digits, n):
    if n != 10:
        return 0
    total = sum((10 - i) * digits[i] for i in range(10))
    return 1 if total % 11 == 0 else 0


def in_range(x, lo, hi):
    return 1 if lo <= x <= hi else 0


def is_leap_year(y):
    if y % 400 == 0:
        return 1
    if y % 100 == 0:
        return 0
    return 1 if y % 4 == 0 else 0


def is_valid_month(m):
    return 1 if 1 <= m <= 12 else 0


def days_in_month(y, m):
    if m == 2:
        return 29 if is_leap_year(y) else 28
    if m in (4, 6, 9, 11):
        return 30
    if 1 <= m <= 12:
        return 31
    return 0


def is_valid_day(y, m, d):
    if not is_valid_month(m):
        return 0
    if d < 1 or d > days_in_month(y, m):
        return 0
    return 1


def is_ascii_digit(c):
    return 1 if "0" <= c <= "9" else 0


def is_ascii_alpha(c):
    return 1 if ("A" <= c <= "Z") or ("a" <= c <= "z") else 0


def all_digits(s):
    if not s:
        return 0
    for c in s:
        if c < "0" or c > "9":
            return 0
    return 1


def password_score(s):
    score = 0
    if len(s) >= 8:
        score += 1
    has_digit = has_lower = has_upper = 0
    for c in s:
        if "0" <= c <= "9":
            has_digit = 1
        elif "a" <= c <= "z":
            has_lower = 1
        elif "A" <= c <= "Z":
            has_upper = 1
    return score + has_digit + has_lower + has_upper


def is_hex_string(s):
    if not s:
        return 0
    for c in s:
        ok = ("0" <= c <= "9") or ("a" <= c <= "f") or ("A" <= c <= "F")
        if not ok:
            return 0
    return 1


def checksum_mod10(digits, n):
    return sum(digits[i] for i in range(n)) % 10


def valid_percentage(x):
    return 1 if 0 <= x <= 100 else 0


def is_valid_rgb(r, g, b):
    return 1 if 0 <= r <= 255 and 0 <= g <= 255 and 0 <= b <= 255 else 0


def even_parity(bits, n):
    ones = sum(1 for i in range(n) if bits[i] == 1)
    return 1 if ones % 2 == 0 else 0


if __name__ == "__main__":
    card = [7, 9, 9, 2, 7, 3, 9, 8, 7, 1, 3]
    isbn = [0, 3, 0, 6, 4, 0, 6, 1, 5, 2]
    bits = [1, 0, 1, 1, 0, 0]
    assert luhn_check(card, 11) == 1
    assert is_valid_isbn10(isbn, 10) == 1
    assert in_range(5, 1, 10) == 1
    assert is_leap_year(2000) == 1
    assert is_leap_year(1900) == 0
    assert is_valid_month(13) == 0
    assert is_valid_day(2021, 2, 29) == 0
    assert is_valid_day(2020, 2, 29) == 1
    assert is_ascii_digit("7") == 1
    assert is_ascii_alpha("Q") == 1
    assert all_digits("12345") == 1
    assert password_score("Abcdef12") == 4
    assert is_hex_string("1aF") == 1
    assert checksum_mod10(card, 11) == 5
    assert valid_percentage(101) == 0
    assert is_valid_rgb(255, 128, 0) == 1
    assert even_parity(bits, 6) == 0
    print("ok")
