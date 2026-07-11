"""Cross-language hashing suite (Python). Deterministic integer hashes over an
int byte array (values 0..255) plus its length. Arithmetic only — no bitwise
operators, to stay algorithm-identical to Lullaby. Moduli and products are kept
below 2^53 so JavaScript doubles stay exact too."""


def djb2_hash(a: list[int], n: int) -> int:
    m = 1000000007
    h = 5381
    for i in range(n):
        h = (h * 33 + a[i]) % m
    return h


def fnv1a_arithmetic(a: list[int], n: int) -> int:
    m = 100000007
    prime = 16777619
    h = 2166136261 % m
    for i in range(n):
        h = ((h + a[i]) * prime) % m
    return h


def sum_hash(a: list[int], n: int) -> int:
    total = 0
    for i in range(n):
        total += a[i]
    return total


def poly_hash(a: list[int], n: int, base: int, modulus: int) -> int:
    h = 0
    for i in range(n):
        h = (h * base + a[i]) % modulus
    return h


def rolling_hash(a: list[int], n: int, modulus: int) -> int:
    h = 0
    for i in range(n):
        h = (h * 256 + a[i]) % modulus
    return h


def sdbm_hash(a: list[int], n: int) -> int:
    m = 1000000007
    h = 0
    for i in range(n):
        h = (a[i] + h * 65599) % m
    return h


def mod_sum_hash(a: list[int], n: int, m: int) -> int:
    h = 0
    for i in range(n):
        h = (h + a[i]) % m
    return h


def weighted_pos_hash(a: list[int], n: int) -> int:
    m = 1000000007
    h = 0
    for i in range(n):
        h = (h + (i + 1) * a[i]) % m
    return h


def xor_free_checksum(a: list[int], n: int) -> int:
    total = 0
    for i in range(n):
        total += a[i]
    return total % 256


def product_hash(a: list[int], n: int, m: int) -> int:
    h = 1
    for i in range(n):
        h = (h * (a[i] + 1)) % m
    return h


def rotate_hash(a: list[int], n: int) -> int:
    m = 1073741824
    half = 536870912
    h = 0
    for i in range(n):
        carry = h // half
        h = (h * 2) % m + carry
        h = (h + a[i]) % m
    return h


def pearson_like(a: list[int], n: int) -> int:
    t = [7, 3, 11, 15, 0, 9, 5, 13, 1, 14, 6, 10, 2, 12, 4, 8]
    h = 0
    for i in range(n):
        h = t[(h + a[i]) % 16]
    return h


def length_hash(a: list[int], n: int) -> int:
    m = 1000000007
    if n <= 0:
        return 0
    return (n * 2654435761 + a[0]) % m


def first_last_hash(a: list[int], n: int) -> int:
    if n <= 0:
        return 0
    return a[0] * 257 + a[n - 1]


def midpoint_hash(a: list[int], n: int) -> int:
    if n <= 0:
        return 0
    return a[n // 2] * 33 + n


def digit_hash(x: int) -> int:
    if x < 0:
        x = -x
    if x == 0:
        return 0
    m = 1000000007
    h = 7
    while x > 0:
        h = (h * 31 + x % 10) % m
        x //= 10
    return h


def pair_hash(a: int, b: int) -> int:
    s = a + b
    return s * (s + 1) // 2 + b


def main() -> None:
    data = [72, 101, 108, 108, 111]
    print("djb2_hash=" + str(djb2_hash(data, 5)))
    print("fnv1a_arithmetic=" + str(fnv1a_arithmetic(data, 5)))
    print("sum_hash=" + str(sum_hash(data, 5)))
    print("poly_hash=" + str(poly_hash(data, 5, 31, 1000000007)))
    print("rolling_hash=" + str(rolling_hash(data, 5, 1000000007)))
    print("sdbm_hash=" + str(sdbm_hash(data, 5)))
    print("mod_sum_hash=" + str(mod_sum_hash(data, 5, 97)))
    print("weighted_pos_hash=" + str(weighted_pos_hash(data, 5)))
    print("xor_free_checksum=" + str(xor_free_checksum(data, 5)))
    print("product_hash=" + str(product_hash(data, 5, 1000000007)))
    print("rotate_hash=" + str(rotate_hash(data, 5)))
    print("pearson_like=" + str(pearson_like(data, 5)))
    print("length_hash=" + str(length_hash(data, 5)))
    print("first_last_hash=" + str(first_last_hash(data, 5)))
    print("midpoint_hash=" + str(midpoint_hash(data, 5)))
    print("digit_hash=" + str(digit_hash(1234)))
    print("pair_hash=" + str(pair_hash(17, 42)))


if __name__ == "__main__":
    main()
