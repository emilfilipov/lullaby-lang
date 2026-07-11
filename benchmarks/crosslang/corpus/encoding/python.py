"""Cross-language encoding suite (Python). Checksums, ciphers, bit tricks over
int byte arrays (values 0..255). Arithmetic only — no bitwise operators, to stay
algorithm-identical to Lullaby."""


def sum_bytes(a: list[int], n: int) -> int:
    total = 0
    for i in range(n):
        total += a[i]
    return total


def add_checksum_mod256(a: list[int], n: int) -> int:
    total = 0
    for i in range(n):
        total += a[i]
    return total % 256


def fletcher16(a: list[int], n: int) -> int:
    sum1 = 0
    sum2 = 0
    for i in range(n):
        sum1 = (sum1 + a[i]) % 255
        sum2 = (sum2 + sum1) % 255
    return sum2 * 256 + sum1


def adler32_small(a: list[int], n: int) -> int:
    s1 = 1
    s2 = 0
    for i in range(n):
        s1 = (s1 + a[i]) % 65521
        s2 = (s2 + s1) % 65521
    return s2 * 65536 + s1


def caesar_encrypt_val(c: int, k: int) -> int:
    if 97 <= c <= 122:
        return 97 + (c - 97 + k % 26) % 26
    return c


def caesar_decrypt_val(c: int, k: int) -> int:
    if 97 <= c <= 122:
        return 97 + (c - 97 + 26 - k % 26) % 26
    return c


def rot13_val(c: int) -> int:
    if 97 <= c <= 122:
        return 97 + (c - 97 + 13) % 26
    if 65 <= c <= 90:
        return 65 + (c - 65 + 13) % 26
    return c


def count_set_bits(x: int) -> int:
    count = 0
    while x > 0:
        count += x % 2
        x //= 2
    return count


def to_binary_length(x: int) -> int:
    if x == 0:
        return 1
    length = 0
    while x > 0:
        length += 1
        x //= 2
    return length


def hex_digit_value(c: int) -> int:
    if 48 <= c <= 57:
        return c - 48
    if 97 <= c <= 102:
        return c - 97 + 10
    return -1


def nibble_to_hex_code(v: int) -> int:
    if v < 10:
        return 48 + v
    return 97 + v - 10


def luhn_from_array(a: list[int], n: int) -> int:
    total = 0
    for i in range(n):
        d = a[n - 1 - i]
        if i % 2 == 1:
            d = d * 2
            if d > 9:
                d = d - 9
        total += d
    return 1 if total % 10 == 0 else 0


def parity_bit(a: list[int], n: int) -> int:
    ones = 0
    for i in range(n):
        ones += a[i]
    return ones % 2


def crc8_simple(a: list[int], n: int) -> int:
    crc = 0
    for i in range(n):
        crc = (crc + a[i]) % 256
        crc = (crc * 2) % 256 + crc // 128
    return crc


def digit_product(x: int) -> int:
    if x < 0:
        x = -x
    if x == 0:
        return 0
    p = 1
    while x > 0:
        p = p * (x % 10)
        x //= 10
    return p


def main() -> None:
    data = [72, 101, 108, 108, 111]
    print("sum_bytes=" + str(sum_bytes(data, 5)))
    print("add_checksum_mod256=" + str(add_checksum_mod256(data, 5)))
    print("fletcher16=" + str(fletcher16(data, 5)))
    print("adler32_small=" + str(adler32_small(data, 5)))
    print("caesar_encrypt_val=" + str(caesar_encrypt_val(104, 3)))
    print("caesar_decrypt_val=" + str(caesar_decrypt_val(107, 3)))
    print("rot13_val=" + str(rot13_val(97)))
    print("count_set_bits=" + str(count_set_bits(181)))
    print("to_binary_length=" + str(to_binary_length(181)))
    print("hex_digit_value=" + str(hex_digit_value(102)))
    print("nibble_to_hex_code=" + str(nibble_to_hex_code(12)))
    card = [7, 9, 9, 2, 7, 3, 9, 8, 7, 1, 3]
    print("luhn_from_array=" + str(luhn_from_array(card, 11)))
    bits = [1, 0, 1, 1, 0]
    print("parity_bit=" + str(parity_bit(bits, 5)))
    print("crc8_simple=" + str(crc8_simple(data, 5)))
    print("digit_product=" + str(digit_product(1234)))


if __name__ == "__main__":
    main()
