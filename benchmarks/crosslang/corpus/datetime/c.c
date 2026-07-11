/* Cross-language datetime suite (C). Pure int64 calendar/date arithmetic.
   No datetime library — the math is implemented directly for a fair comparison.
   Conventions: is_weekend expects dow 0=Sun..6=Sat; zeller_dow returns
   0=Sun..6=Sat; days_from_civil counts days since 1970-01-01 (Hinnant). */
#include <stdio.h>
#include <stdint.h>

int64_t is_leap_year(int64_t y) {
    return (y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)) ? 1 : 0;
}

int64_t days_in_month(int64_t y, int64_t m) {
    if (m == 2) return is_leap_year(y) ? 29 : 28;
    if (m == 4 || m == 6 || m == 9 || m == 11) return 30;
    return 31;
}

int64_t day_of_year(int64_t y, int64_t m, int64_t d) {
    int64_t total = d;
    for (int64_t mm = 1; mm < m; mm++) total += days_in_month(y, mm);
    return total;
}

int64_t is_weekend(int64_t dow) {
    return (dow == 0 || dow == 6) ? 1 : 0;
}

int64_t zeller_dow(int64_t y, int64_t m, int64_t d) {
    if (m < 3) { m += 12; y -= 1; }
    int64_t k = y % 100;
    int64_t j = y / 100;
    int64_t h = (d + 13 * (m + 1) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    return (h + 6) % 7;
}

int64_t days_from_civil(int64_t y, int64_t m, int64_t d) {
    y -= m <= 2;
    int64_t era = (y >= 0 ? y : y - 399) / 400;
    int64_t yoe = y - era * 400;
    int64_t doy = (153 * (m > 2 ? m - 3 : m + 9) + 2) / 5 + d - 1;
    int64_t doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    return era * 146097 + doe - 719468;
}

int64_t year_from_days(int64_t z) {
    z += 719468;
    int64_t era = (z >= 0 ? z : z - 146096) / 146097;
    int64_t doe = z - era * 146097;
    int64_t yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    int64_t y = yoe + era * 400;
    int64_t doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    int64_t mp = (5 * doy + 2) / 153;
    int64_t m = mp < 10 ? mp + 3 : mp - 9;
    return y + (m <= 2);
}

int64_t days_between(int64_t y1, int64_t m1, int64_t d1,
                     int64_t y2, int64_t m2, int64_t d2) {
    return days_from_civil(y2, m2, d2) - days_from_civil(y1, m1, d1);
}

int64_t add_days_year(int64_t y, int64_t m, int64_t d, int64_t n) {
    return year_from_days(days_from_civil(y, m, d) + n);
}

int64_t hours_to_minutes(int64_t h) { return h * 60; }
int64_t minutes_to_seconds(int64_t m) { return m * 60; }
int64_t seconds_in_days(int64_t d) { return d * 86400; }

int64_t quarter_of_month(int64_t m) { return (m - 1) / 3 + 1; }

int64_t is_valid_time(int64_t h, int64_t mi, int64_t s) {
    return (h >= 0 && h <= 23 && mi >= 0 && mi <= 59 && s >= 0 && s <= 59) ? 1 : 0;
}

int64_t clock_add_minutes(int64_t h, int64_t mi, int64_t add) {
    return ((h * 60 + mi + add) % 1440) / 60;
}

int64_t age_years(int64_t yb, int64_t mb, int64_t db,
                  int64_t yn, int64_t mn, int64_t dn) {
    int64_t years = yn - yb;
    if (mn < mb || (mn == mb && dn < db)) years -= 1;
    return years;
}

int main(void) {
    printf("is_leap_year(2000)=%lld\n", (long long)is_leap_year(2000));
    printf("days_in_month(2024,2)=%lld\n", (long long)days_in_month(2024, 2));
    printf("day_of_year(2024,3,1)=%lld\n", (long long)day_of_year(2024, 3, 1));
    printf("is_weekend(6)=%lld\n", (long long)is_weekend(6));
    printf("zeller_dow(2024,1,1)=%lld\n", (long long)zeller_dow(2024, 1, 1));
    printf("days_from_civil(2000,1,1)=%lld\n", (long long)days_from_civil(2000, 1, 1));
    printf("days_between=%lld\n", (long long)days_between(2020, 1, 1, 2024, 1, 1));
    printf("add_days_year(2023,12,20,30)=%lld\n", (long long)add_days_year(2023, 12, 20, 30));
    printf("hours_to_minutes(3)=%lld\n", (long long)hours_to_minutes(3));
    printf("minutes_to_seconds(5)=%lld\n", (long long)minutes_to_seconds(5));
    printf("seconds_in_days(2)=%lld\n", (long long)seconds_in_days(2));
    printf("quarter_of_month(7)=%lld\n", (long long)quarter_of_month(7));
    printf("is_valid_time(23,59,59)=%lld\n", (long long)is_valid_time(23, 59, 59));
    printf("clock_add_minutes(23,30,90)=%lld\n", (long long)clock_add_minutes(23, 30, 90));
    printf("age_years(1990,6,15,2024,6,14)=%lld\n", (long long)age_years(1990, 6, 15, 2024, 6, 14));
    return 0;
}
