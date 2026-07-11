"""Cross-language datetime suite (Python). Pure int calendar/date arithmetic.

No datetime library — the math is implemented directly for a fair comparison.
Conventions: is_weekend expects dow 0=Sun..6=Sat; zeller_dow returns
0=Sun..6=Sat; days_from_civil counts days since 1970-01-01 (Hinnant).
"""


def is_leap_year(y: int) -> int:
    return 1 if y % 4 == 0 and (y % 100 != 0 or y % 400 == 0) else 0


def days_in_month(y: int, m: int) -> int:
    if m == 2:
        return 29 if is_leap_year(y) else 28
    if m in (4, 6, 9, 11):
        return 30
    return 31


def day_of_year(y: int, m: int, d: int) -> int:
    total = d
    for mm in range(1, m):
        total += days_in_month(y, mm)
    return total


def is_weekend(dow: int) -> int:
    return 1 if dow == 0 or dow == 6 else 0


def zeller_dow(y: int, m: int, d: int) -> int:
    if m < 3:
        m += 12
        y -= 1
    k = y % 100
    j = y // 100
    h = (d + 13 * (m + 1) // 5 + k + k // 4 + j // 4 + 5 * j) % 7
    return (h + 6) % 7


def days_from_civil(y: int, m: int, d: int) -> int:
    y -= m <= 2
    era = (y if y >= 0 else y - 399) // 400
    yoe = y - era * 400
    doy = (153 * (m - 3 if m > 2 else m + 9) + 2) // 5 + d - 1
    doe = yoe * 365 + yoe // 4 - yoe // 100 + doy
    return era * 146097 + doe - 719468


def year_from_days(z: int) -> int:
    z += 719468
    era = (z if z >= 0 else z - 146096) // 146097
    doe = z - era * 146097
    yoe = (doe - doe // 1460 + doe // 36524 - doe // 146096) // 365
    y = yoe + era * 400
    doy = doe - (365 * yoe + yoe // 4 - yoe // 100)
    mp = (5 * doy + 2) // 153
    m = mp + 3 if mp < 10 else mp - 9
    return y + (m <= 2)


def days_between(y1: int, m1: int, d1: int, y2: int, m2: int, d2: int) -> int:
    return days_from_civil(y2, m2, d2) - days_from_civil(y1, m1, d1)


def add_days_year(y: int, m: int, d: int, n: int) -> int:
    return year_from_days(days_from_civil(y, m, d) + n)


def hours_to_minutes(h: int) -> int:
    return h * 60


def minutes_to_seconds(m: int) -> int:
    return m * 60


def seconds_in_days(d: int) -> int:
    return d * 86400


def quarter_of_month(m: int) -> int:
    return (m - 1) // 3 + 1


def is_valid_time(h: int, mi: int, s: int) -> int:
    return 1 if 0 <= h <= 23 and 0 <= mi <= 59 and 0 <= s <= 59 else 0


def clock_add_minutes(h: int, mi: int, add: int) -> int:
    return (h * 60 + mi + add) % 1440 // 60


def age_years(yb: int, mb: int, db: int, yn: int, mn: int, dn: int) -> int:
    years = yn - yb
    if mn < mb or (mn == mb and dn < db):
        years -= 1
    return years


def main() -> None:
    print("is_leap_year(2000)=" + str(is_leap_year(2000)))
    print("days_in_month(2024,2)=" + str(days_in_month(2024, 2)))
    print("day_of_year(2024,3,1)=" + str(day_of_year(2024, 3, 1)))
    print("is_weekend(6)=" + str(is_weekend(6)))
    print("zeller_dow(2024,1,1)=" + str(zeller_dow(2024, 1, 1)))
    print("days_from_civil(2000,1,1)=" + str(days_from_civil(2000, 1, 1)))
    print("days_between=" + str(days_between(2020, 1, 1, 2024, 1, 1)))
    print("add_days_year(2023,12,20,30)=" + str(add_days_year(2023, 12, 20, 30)))
    print("hours_to_minutes(3)=" + str(hours_to_minutes(3)))
    print("minutes_to_seconds(5)=" + str(minutes_to_seconds(5)))
    print("seconds_in_days(2)=" + str(seconds_in_days(2)))
    print("quarter_of_month(7)=" + str(quarter_of_month(7)))
    print("is_valid_time(23,59,59)=" + str(is_valid_time(23, 59, 59)))
    print("clock_add_minutes(23,30,90)=" + str(clock_add_minutes(23, 30, 90)))
    print("age_years(1990,6,15,2024,6,14)=" + str(age_years(1990, 6, 15, 2024, 6, 14)))


if __name__ == "__main__":
    main()
