// Cross-language units suite (JavaScript). Unit/quantity conversions in integers.

function minutes_to_seconds(m) {
    return m * 60;
}

function hours_to_minutes(h) {
    return h * 60;
}

function days_to_hours(d) {
    return d * 24;
}

function weeks_to_days(w) {
    return w * 7;
}

function bytes_to_kib(b) {
    return Math.trunc(b / 1024);
}

function kib_to_bytes(k) {
    return k * 1024;
}

function mib_to_bytes(m) {
    return m * 1024 * 1024;
}

function meters_to_cm(m) {
    return m * 100;
}

function cm_to_mm(c) {
    return c * 10;
}

function km_to_meters(k) {
    return k * 1000;
}

function feet_to_inches(f) {
    return f * 12;
}

function inches_to_cm_x10(i) {
    return Math.trunc(i * 254 / 10);
}

function pounds_to_ounces(p) {
    return p * 16;
}

function gallons_to_pints(g) {
    return g * 8;
}

function liters_to_ml(l) {
    return l * 1000;
}

function degrees_to_gradians(d) {
    return Math.trunc(d * 10 / 9);
}

function radians_milli_to_degrees(mr) {
    return Math.trunc(mr * 180 / 3142);
}

function seconds_to_hms_hours(s) {
    return Math.trunc(s / 3600);
}

function main() {
    console.log("minutes_to_seconds=" + minutes_to_seconds(5));
    console.log("hours_to_minutes=" + hours_to_minutes(3));
    console.log("days_to_hours=" + days_to_hours(2));
    console.log("weeks_to_days=" + weeks_to_days(3));
    console.log("bytes_to_kib=" + bytes_to_kib(4096));
    console.log("kib_to_bytes=" + kib_to_bytes(4));
    console.log("mib_to_bytes=" + mib_to_bytes(2));
    console.log("meters_to_cm=" + meters_to_cm(7));
    console.log("cm_to_mm=" + cm_to_mm(12));
    console.log("km_to_meters=" + km_to_meters(5));
    console.log("feet_to_inches=" + feet_to_inches(6));
    console.log("inches_to_cm_x10=" + inches_to_cm_x10(10));
    console.log("pounds_to_ounces=" + pounds_to_ounces(3));
    console.log("gallons_to_pints=" + gallons_to_pints(2));
    console.log("liters_to_ml=" + liters_to_ml(3));
    console.log("degrees_to_gradians=" + degrees_to_gradians(90));
    console.log("radians_milli_to_degrees=" + radians_milli_to_degrees(3142));
    console.log("seconds_to_hms_hours=" + seconds_to_hms_hours(7325));
}

main();
