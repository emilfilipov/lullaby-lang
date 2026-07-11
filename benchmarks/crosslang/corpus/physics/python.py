# Cross-language physics suite (Python). Integer physics & unit conversions.
# All math is scaled-integer (no floats): temperatures are tenths (x10),
# distances/speeds use fixed scale factors, energy uses g = 10 m/s^2.


def distance_uniform(v, t):
    return v * t


def distance_accel(v, t, a):
    return v * t + a * t * t // 2


def final_velocity(v, a, t):
    return v + a * t


def kinetic_energy_x2(m, v):
    return m * v * v


def momentum(m, v):
    return m * v


def force(m, a):
    return m * a


def work(f, d):
    return f * d


def power_watts(work, t):
    return work // t


def pressure(f, area):
    if area == 0:
        return 0
    return f // area


def density(mass, vol):
    return mass // vol


def celsius_to_fahrenheit_x10(c):
    return c * 18 + 320


def fahrenheit_to_celsius_x10(f):
    return (f - 32) * 100 // 18


def celsius_to_kelvin_x10(c):
    return c * 10 + 2732


def km_to_miles_scaled(km):
    return km * 621 // 1000


def miles_to_km_scaled(mi):
    return mi * 1609 // 1000


def mps_to_kmh(mps):
    return mps * 36 // 10


def gravity_fall_dist(t):
    return 5 * t * t


def potential_energy(m, h):
    return m * 10 * h


def main():
    print("distance_uniform=" + str(distance_uniform(60, 2)))
    print("distance_accel=" + str(distance_accel(10, 4, 3)))
    print("final_velocity=" + str(final_velocity(5, 3, 4)))
    print("kinetic_energy_x2=" + str(kinetic_energy_x2(4, 5)))
    print("momentum=" + str(momentum(8, 3)))
    print("force=" + str(force(10, 9)))
    print("work=" + str(work(50, 4)))
    print("power_watts=" + str(power_watts(1000, 4)))
    print("pressure=" + str(pressure(200, 4)))
    print("density=" + str(density(1000, 8)))
    print("celsius_to_fahrenheit_x10=" + str(celsius_to_fahrenheit_x10(25)))
    print("fahrenheit_to_celsius_x10=" + str(fahrenheit_to_celsius_x10(77)))
    print("celsius_to_kelvin_x10=" + str(celsius_to_kelvin_x10(25)))
    print("km_to_miles_scaled=" + str(km_to_miles_scaled(100)))
    print("miles_to_km_scaled=" + str(miles_to_km_scaled(100)))
    print("mps_to_kmh=" + str(mps_to_kmh(10)))
    print("gravity_fall_dist=" + str(gravity_fall_dist(3)))
    print("potential_energy=" + str(potential_energy(2, 10)))


if __name__ == "__main__":
    main()
