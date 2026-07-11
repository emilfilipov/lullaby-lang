"""Cross-language matrix suite (Python). Dense matrix operations over a flat
row-major list: an n*n matrix is passed as a list plus its order n, with
element (r, c) stored at m[r * n + c]."""


def diagonal_sum_main(m: list, n: int) -> int:
    total = 0
    for i in range(n):
        total += m[i * n + i]
    return total


def trace(m: list, n: int) -> int:
    return diagonal_sum_main(m, n)


def diagonal_sum_anti(m: list, n: int) -> int:
    total = 0
    for i in range(n):
        total += m[i * n + (n - 1 - i)]
    return total


def row_sum(m: list, n: int, r: int) -> int:
    total = 0
    for c in range(n):
        total += m[r * n + c]
    return total


def col_sum(m: list, n: int, c: int) -> int:
    total = 0
    for r in range(n):
        total += m[r * n + c]
    return total


def row_sum_max(m: list, n: int) -> int:
    best = row_sum(m, n, 0)
    for r in range(1, n):
        s = row_sum(m, n, r)
        if s > best:
            best = s
    return best


def col_sum_max(m: list, n: int) -> int:
    best = col_sum(m, n, 0)
    for c in range(1, n):
        s = col_sum(m, n, c)
        if s > best:
            best = s
    return best


def is_symmetric(m: list, n: int) -> int:
    for i in range(n):
        for j in range(n):
            if m[i * n + j] != m[j * n + i]:
                return 0
    return 1


def transpose_checksum(m: list, n: int) -> int:
    total = 0
    for r in range(n):
        for c in range(n):
            i = r * n + c
            total += i * m[c * n + r]
    return total


def matrix_add_checksum(a: list, b: list, n: int) -> int:
    total = 0
    for i in range(n * n):
        total += i * (a[i] + b[i])
    return total


def scalar_mul_checksum(m: list, n: int, k: int) -> int:
    total = 0
    for i in range(n * n):
        total += i * (m[i] * k)
    return total


def is_identity(m: list, n: int) -> int:
    for i in range(n):
        for j in range(n):
            if i == j:
                if m[i * n + j] != 1:
                    return 0
            elif m[i * n + j] != 0:
                return 0
    return 1


def matrix_mul_trace(a: list, b: list, n: int) -> int:
    total = 0
    for i in range(n):
        for k in range(n):
            total += a[i * n + k] * b[k * n + i]
    return total


def main_diag_product(m: list, n: int) -> int:
    product = 1
    for i in range(n):
        product *= m[i * n + i]
    return product


def max_element(m: list, n: int) -> int:
    best = m[0]
    for i in range(1, n * n):
        if m[i] > best:
            best = m[i]
    return best


def min_element(m: list, n: int) -> int:
    best = m[0]
    for i in range(1, n * n):
        if m[i] < best:
            best = m[i]
    return best


def determinant_2x2(m: list) -> int:
    return m[0] * m[3] - m[1] * m[2]


def determinant_3x3(m: list) -> int:
    return (m[0] * (m[4] * m[8] - m[5] * m[7])
            - m[1] * (m[3] * m[8] - m[5] * m[6])
            + m[2] * (m[3] * m[7] - m[4] * m[6]))


def main() -> None:
    m = [1, 2, 3, 4, 5, 6, 7, 8, 9]
    sym = [1, 2, 3, 2, 5, 6, 3, 6, 9]
    ident = [1, 0, 0, 0, 1, 0, 0, 0, 1]
    b = [9, 8, 7, 6, 5, 4, 3, 2, 1]
    d2 = [1, 2, 3, 4]
    print("trace=" + str(trace(m, 3)))
    print("diagonal_sum_main=" + str(diagonal_sum_main(m, 3)))
    print("diagonal_sum_anti=" + str(diagonal_sum_anti(m, 3)))
    print("row_sum=" + str(row_sum(m, 3, 1)))
    print("col_sum=" + str(col_sum(m, 3, 2)))
    print("row_sum_max=" + str(row_sum_max(m, 3)))
    print("col_sum_max=" + str(col_sum_max(m, 3)))
    print("is_symmetric=" + str(is_symmetric(sym, 3)))
    print("transpose_checksum=" + str(transpose_checksum(m, 3)))
    print("matrix_add_checksum=" + str(matrix_add_checksum(m, b, 3)))
    print("scalar_mul_checksum=" + str(scalar_mul_checksum(m, 3, 2)))
    print("is_identity=" + str(is_identity(ident, 3)))
    print("matrix_mul_trace=" + str(matrix_mul_trace(m, b, 3)))
    print("main_diag_product=" + str(main_diag_product(m, 3)))
    print("max_element=" + str(max_element(m, 3)))
    print("min_element=" + str(min_element(m, 3)))
    print("determinant_2x2=" + str(determinant_2x2(d2)))
    print("determinant_3x3=" + str(determinant_3x3(m)))


if __name__ == "__main__":
    main()
