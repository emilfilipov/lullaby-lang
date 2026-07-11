// Cross-language datetime suite (JavaScript). Pure integer calendar/date
// arithmetic. No Date library — the math is implemented directly for a fair
// comparison. Conventions: is_weekend expects dow 0=Sun..6=Sat; zeller_dow
// returns 0=Sun..6=Sat; days_from_civil counts days since 1970-01-01 (Hinnant).

function is_leap_year(y) {
  return y % 4 === 0 && (y % 100 !== 0 || y % 400 === 0) ? 1 : 0;
}

function days_in_month(y, m) {
  if (m === 2) {
    return is_leap_year(y) === 1 ? 29 : 28;
  }
  if (m === 4 || m === 6 || m === 9 || m === 11) {
    return 30;
  }
  return 31;
}

function day_of_year(y, m, d) {
  let total = d;
  for (let mm = 1; mm < m; mm++) {
    total += days_in_month(y, mm);
  }
  return total;
}

function is_weekend(dow) {
  return dow === 0 || dow === 6 ? 1 : 0;
}

function zeller_dow(y, m, d) {
  if (m < 3) {
    m += 12;
    y -= 1;
  }
  const k = y % 100;
  const j = Math.trunc(y / 100);
  const h = (d + Math.trunc((13 * (m + 1)) / 5) + k + Math.trunc(k / 4) + Math.trunc(j / 4) + 5 * j) % 7;
  return (h + 6) % 7;
}

function days_from_civil(y, m, d) {
  y -= m <= 2 ? 1 : 0;
  const era = Math.trunc((y >= 0 ? y : y - 399) / 400);
  const yoe = y - era * 400;
  const doy = Math.trunc((153 * (m > 2 ? m - 3 : m + 9) + 2) / 5) + d - 1;
  const doe = yoe * 365 + Math.trunc(yoe / 4) - Math.trunc(yoe / 100) + doy;
  return era * 146097 + doe - 719468;
}

function year_from_days(z) {
  z += 719468;
  const era = Math.trunc((z >= 0 ? z : z - 146096) / 146097);
  const doe = z - era * 146097;
  const yoe = Math.trunc((doe - Math.trunc(doe / 1460) + Math.trunc(doe / 36524) - Math.trunc(doe / 146096)) / 365);
  const y = yoe + era * 400;
  const doy = doe - (365 * yoe + Math.trunc(yoe / 4) - Math.trunc(yoe / 100));
  const mp = Math.trunc((5 * doy + 2) / 153);
  const m = mp < 10 ? mp + 3 : mp - 9;
  return y + (m <= 2 ? 1 : 0);
}

function days_between(y1, m1, d1, y2, m2, d2) {
  return days_from_civil(y2, m2, d2) - days_from_civil(y1, m1, d1);
}

function add_days_year(y, m, d, n) {
  return year_from_days(days_from_civil(y, m, d) + n);
}

function hours_to_minutes(h) {
  return h * 60;
}

function minutes_to_seconds(m) {
  return m * 60;
}

function seconds_in_days(d) {
  return d * 86400;
}

function quarter_of_month(m) {
  return Math.trunc((m - 1) / 3) + 1;
}

function is_valid_time(h, mi, s) {
  return h >= 0 && h <= 23 && mi >= 0 && mi <= 59 && s >= 0 && s <= 59 ? 1 : 0;
}

function clock_add_minutes(h, mi, add) {
  return Math.trunc(((h * 60 + mi + add) % 1440) / 60);
}

function age_years(yb, mb, db, yn, mn, dn) {
  let years = yn - yb;
  if (mn < mb || (mn === mb && dn < db)) {
    years -= 1;
  }
  return years;
}

function main() {
  console.log("is_leap_year(2000)=" + is_leap_year(2000));
  console.log("days_in_month(2024,2)=" + days_in_month(2024, 2));
  console.log("day_of_year(2024,3,1)=" + day_of_year(2024, 3, 1));
  console.log("is_weekend(6)=" + is_weekend(6));
  console.log("zeller_dow(2024,1,1)=" + zeller_dow(2024, 1, 1));
  console.log("days_from_civil(2000,1,1)=" + days_from_civil(2000, 1, 1));
  console.log("days_between=" + days_between(2020, 1, 1, 2024, 1, 1));
  console.log("add_days_year(2023,12,20,30)=" + add_days_year(2023, 12, 20, 30));
  console.log("hours_to_minutes(3)=" + hours_to_minutes(3));
  console.log("minutes_to_seconds(5)=" + minutes_to_seconds(5));
  console.log("seconds_in_days(2)=" + seconds_in_days(2));
  console.log("quarter_of_month(7)=" + quarter_of_month(7));
  console.log("is_valid_time(23,59,59)=" + is_valid_time(23, 59, 59));
  console.log("clock_add_minutes(23,30,90)=" + clock_add_minutes(23, 30, 90));
  console.log("age_years(1990,6,15,2024,6,14)=" + age_years(1990, 6, 15, 2024, 6, 14));
}

main();
