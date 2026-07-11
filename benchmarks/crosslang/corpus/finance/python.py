# Cross-language finance suite (Python). Money/business math in integer cents.


def add_tax(cents, rate_bp):
    return cents + cents * rate_bp // 10000


def apply_discount(cents, pct):
    return cents - cents * pct // 100


def tip_amount(cents, pct):
    return cents * pct // 100


def split_bill(cents, people):
    return cents // people


def split_remainder(cents, people):
    return cents % people


def simple_interest(principal, rate_bp, years):
    return principal * rate_bp * years // 10000


def compound_interest(principal, rate_bp, years):
    amount = principal
    for _ in range(years):
        amount = amount + amount * rate_bp // 10000
    return amount


def monthly_payment_flat(principal, months):
    return principal // months


def cents_to_dollars(cents):
    return cents // 100


def cents_part(cents):
    return cents % 100


def percent_of(part, whole):
    if whole == 0:
        return 0
    return part * 100 // whole


def markup(cost, pct):
    return cost + cost * pct // 100


def margin_pct(cost, price):
    if price == 0:
        return 0
    return (price - cost) * 100 // price


def round_to_nearest(cents, step):
    return (cents + step // 2) // step * step


def future_value_years(principal, rate_bp, years):
    amount = principal
    y = 0
    while y < years:
        amount += amount * rate_bp // 10000
        y += 1
    return amount


def main():
    print("add_tax=" + str(add_tax(1000, 825)))
    print("apply_discount=" + str(apply_discount(1000, 15)))
    print("tip_amount=" + str(tip_amount(4200, 18)))
    print("split_bill=" + str(split_bill(10000, 3)))
    print("split_remainder=" + str(split_remainder(10000, 3)))
    print("simple_interest=" + str(simple_interest(100000, 500, 3)))
    print("compound_interest=" + str(compound_interest(100000, 500, 3)))
    print("monthly_payment_flat=" + str(monthly_payment_flat(120000, 12)))
    print("cents_to_dollars=" + str(cents_to_dollars(12345)))
    print("cents_part=" + str(cents_part(12345)))
    print("percent_of=" + str(percent_of(45, 200)))
    print("markup=" + str(markup(5000, 40)))
    print("margin_pct=" + str(margin_pct(6000, 10000)))
    print("round_to_nearest=" + str(round_to_nearest(1237, 25)))
    print("future_value_years=" + str(future_value_years(100000, 500, 3)))


if __name__ == "__main__":
    main()
