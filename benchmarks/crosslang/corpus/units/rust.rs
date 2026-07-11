// Cross-language units suite (Rust). Unit/quantity conversions in integers.

fn minutes_to_seconds(m: i64) -> i64 {
    m * 60
}

fn hours_to_minutes(h: i64) -> i64 {
    h * 60
}

fn days_to_hours(d: i64) -> i64 {
    d * 24
}

fn weeks_to_days(w: i64) -> i64 {
    w * 7
}

fn bytes_to_kib(b: i64) -> i64 {
    b / 1024
}

fn kib_to_bytes(k: i64) -> i64 {
    k * 1024
}

fn mib_to_bytes(m: i64) -> i64 {
    m * 1024 * 1024
}

fn meters_to_cm(m: i64) -> i64 {
    m * 100
}

fn cm_to_mm(c: i64) -> i64 {
    c * 10
}

fn km_to_meters(k: i64) -> i64 {
    k * 1000
}

fn feet_to_inches(f: i64) -> i64 {
    f * 12
}

fn inches_to_cm_x10(i: i64) -> i64 {
    i * 254 / 10
}

fn pounds_to_ounces(p: i64) -> i64 {
    p * 16
}

fn gallons_to_pints(g: i64) -> i64 {
    g * 8
}

fn liters_to_ml(l: i64) -> i64 {
    l * 1000
}

fn degrees_to_gradians(d: i64) -> i64 {
    d * 10 / 9
}

fn radians_milli_to_degrees(mr: i64) -> i64 {
    mr * 180 / 3142
}

fn seconds_to_hms_hours(s: i64) -> i64 {
    s / 3600
}

fn main() {
    println!("minutes_to_seconds={}", minutes_to_seconds(5));
    println!("hours_to_minutes={}", hours_to_minutes(3));
    println!("days_to_hours={}", days_to_hours(2));
    println!("weeks_to_days={}", weeks_to_days(3));
    println!("bytes_to_kib={}", bytes_to_kib(4096));
    println!("kib_to_bytes={}", kib_to_bytes(4));
    println!("mib_to_bytes={}", mib_to_bytes(2));
    println!("meters_to_cm={}", meters_to_cm(7));
    println!("cm_to_mm={}", cm_to_mm(12));
    println!("km_to_meters={}", km_to_meters(5));
    println!("feet_to_inches={}", feet_to_inches(6));
    println!("inches_to_cm_x10={}", inches_to_cm_x10(10));
    println!("pounds_to_ounces={}", pounds_to_ounces(3));
    println!("gallons_to_pints={}", gallons_to_pints(2));
    println!("liters_to_ml={}", liters_to_ml(3));
    println!("degrees_to_gradians={}", degrees_to_gradians(90));
    println!("radians_milli_to_degrees={}", radians_milli_to_degrees(3142));
    println!("seconds_to_hms_hours={}", seconds_to_hms_hours(7325));
}
