# Cross-language units suite (Python). Unit/quantity conversions in integers.


def minutes_to_seconds(m):
    return m * 60


def hours_to_minutes(h):
    return h * 60


def days_to_hours(d):
    return d * 24


def weeks_to_days(w):
    return w * 7


def bytes_to_kib(b):
    return b // 1024


def kib_to_bytes(k):
    return k * 1024


def mib_to_bytes(m):
    return m * 1024 * 1024


def meters_to_cm(m):
    return m * 100


def cm_to_mm(c):
    return c * 10


def km_to_meters(k):
    return k * 1000


def feet_to_inches(f):
    return f * 12


def inches_to_cm_x10(i):
    return i * 254 // 10


def pounds_to_ounces(p):
    return p * 16


def gallons_to_pints(g):
    return g * 8


def liters_to_ml(l):
    return l * 1000


def degrees_to_gradians(d):
    return d * 10 // 9


def radians_milli_to_degrees(mr):
    return mr * 180 // 3142


def seconds_to_hms_hours(s):
    return s // 3600


def main():
    print("minutes_to_seconds=" + str(minutes_to_seconds(5)))
    print("hours_to_minutes=" + str(hours_to_minutes(3)))
    print("days_to_hours=" + str(days_to_hours(2)))
    print("weeks_to_days=" + str(weeks_to_days(3)))
    print("bytes_to_kib=" + str(bytes_to_kib(4096)))
    print("kib_to_bytes=" + str(kib_to_bytes(4)))
    print("mib_to_bytes=" + str(mib_to_bytes(2)))
    print("meters_to_cm=" + str(meters_to_cm(7)))
    print("cm_to_mm=" + str(cm_to_mm(12)))
    print("km_to_meters=" + str(km_to_meters(5)))
    print("feet_to_inches=" + str(feet_to_inches(6)))
    print("inches_to_cm_x10=" + str(inches_to_cm_x10(10)))
    print("pounds_to_ounces=" + str(pounds_to_ounces(3)))
    print("gallons_to_pints=" + str(gallons_to_pints(2)))
    print("liters_to_ml=" + str(liters_to_ml(3)))
    print("degrees_to_gradians=" + str(degrees_to_gradians(90)))
    print("radians_milli_to_degrees=" + str(radians_milli_to_degrees(3142)))
    print("seconds_to_hms_hours=" + str(seconds_to_hms_hours(7325)))


if __name__ == "__main__":
    main()
