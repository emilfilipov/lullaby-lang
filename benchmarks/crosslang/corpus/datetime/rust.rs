// Cross-language datetime suite (Rust). Pure i64 calendar/date arithmetic.
// No datetime library — the math is implemented directly for a fair comparison.
// Conventions: is_weekend expects dow 0=Sun..6=Sat; zeller_dow returns
// 0=Sun..6=Sat; days_from_civil counts days since 1970-01-01 (Hinnant).

fn is_leap_year(y: i64) -> i64 {
    if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 1 } else { 0 }
}

fn days_in_month(y: i64, m: i64) -> i64 {
    if m == 2 {
        if is_leap_year(y) == 1 { 29 } else { 28 }
    } else if m == 4 || m == 6 || m == 9 || m == 11 {
        30
    } else {
        31
    }
}

fn day_of_year(y: i64, m: i64, d: i64) -> i64 {
    let mut total = d;
    for mm in 1..m {
        total += days_in_month(y, mm);
    }
    total
}

fn is_weekend(dow: i64) -> i64 {
    if dow == 0 || dow == 6 { 1 } else { 0 }
}

fn zeller_dow(mut y: i64, mut m: i64, d: i64) -> i64 {
    if m < 3 {
        m += 12;
        y -= 1;
    }
    let k = y % 100;
    let j = y / 100;
    let h = (d + 13 * (m + 1) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    (h + 6) % 7
}

fn days_from_civil(mut y: i64, m: i64, d: i64) -> i64 {
    y -= (m <= 2) as i64;
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn year_from_days(mut z: i64) -> i64 {
    z += 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    y + (m <= 2) as i64
}

fn days_between(y1: i64, m1: i64, d1: i64, y2: i64, m2: i64, d2: i64) -> i64 {
    days_from_civil(y2, m2, d2) - days_from_civil(y1, m1, d1)
}

fn add_days_year(y: i64, m: i64, d: i64, n: i64) -> i64 {
    year_from_days(days_from_civil(y, m, d) + n)
}

fn hours_to_minutes(h: i64) -> i64 { h * 60 }
fn minutes_to_seconds(m: i64) -> i64 { m * 60 }
fn seconds_in_days(d: i64) -> i64 { d * 86400 }

fn quarter_of_month(m: i64) -> i64 { (m - 1) / 3 + 1 }

fn is_valid_time(h: i64, mi: i64, s: i64) -> i64 {
    if h >= 0 && h <= 23 && mi >= 0 && mi <= 59 && s >= 0 && s <= 59 { 1 } else { 0 }
}

fn clock_add_minutes(h: i64, mi: i64, add: i64) -> i64 {
    ((h * 60 + mi + add) % 1440) / 60
}

fn age_years(yb: i64, mb: i64, db: i64, yn: i64, mn: i64, dn: i64) -> i64 {
    let mut years = yn - yb;
    if mn < mb || (mn == mb && dn < db) {
        years -= 1;
    }
    years
}

fn main() {
    println!("is_leap_year(2000)={}", is_leap_year(2000));
    println!("days_in_month(2024,2)={}", days_in_month(2024, 2));
    println!("day_of_year(2024,3,1)={}", day_of_year(2024, 3, 1));
    println!("is_weekend(6)={}", is_weekend(6));
    println!("zeller_dow(2024,1,1)={}", zeller_dow(2024, 1, 1));
    println!("days_from_civil(2000,1,1)={}", days_from_civil(2000, 1, 1));
    println!("days_between={}", days_between(2020, 1, 1, 2024, 1, 1));
    println!("add_days_year(2023,12,20,30)={}", add_days_year(2023, 12, 20, 30));
    println!("hours_to_minutes(3)={}", hours_to_minutes(3));
    println!("minutes_to_seconds(5)={}", minutes_to_seconds(5));
    println!("seconds_in_days(2)={}", seconds_in_days(2));
    println!("quarter_of_month(7)={}", quarter_of_month(7));
    println!("is_valid_time(23,59,59)={}", is_valid_time(23, 59, 59));
    println!("clock_add_minutes(23,30,90)={}", clock_add_minutes(23, 30, 90));
    println!("age_years(1990,6,15,2024,6,14)={}", age_years(1990, 6, 15, 2024, 6, 14));
}
