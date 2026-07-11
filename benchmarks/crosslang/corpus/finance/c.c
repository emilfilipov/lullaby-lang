/* Cross-language finance suite (C). Money/business math in integer cents. */
#include <stdio.h>
#include <stdint.h>

int64_t add_tax(int64_t cents, int64_t rate_bp) {
    return cents + cents * rate_bp / 10000;
}

int64_t apply_discount(int64_t cents, int64_t pct) {
    return cents - cents * pct / 100;
}

int64_t tip_amount(int64_t cents, int64_t pct) {
    return cents * pct / 100;
}

int64_t split_bill(int64_t cents, int64_t people) {
    return cents / people;
}

int64_t split_remainder(int64_t cents, int64_t people) {
    return cents % people;
}

int64_t simple_interest(int64_t principal, int64_t rate_bp, int64_t years) {
    return principal * rate_bp * years / 10000;
}

int64_t compound_interest(int64_t principal, int64_t rate_bp, int64_t years) {
    int64_t amount = principal;
    for (int64_t y = 0; y < years; y++) {
        amount = amount + amount * rate_bp / 10000;
    }
    return amount;
}

int64_t monthly_payment_flat(int64_t principal, int64_t months) {
    return principal / months;
}

int64_t cents_to_dollars(int64_t cents) {
    return cents / 100;
}

int64_t cents_part(int64_t cents) {
    return cents % 100;
}

int64_t percent_of(int64_t part, int64_t whole) {
    if (whole == 0) return 0;
    return part * 100 / whole;
}

int64_t markup(int64_t cost, int64_t pct) {
    return cost + cost * pct / 100;
}

int64_t margin_pct(int64_t cost, int64_t price) {
    if (price == 0) return 0;
    return (price - cost) * 100 / price;
}

int64_t round_to_nearest(int64_t cents, int64_t step) {
    return (cents + step / 2) / step * step;
}

int64_t future_value_years(int64_t principal, int64_t rate_bp, int64_t years) {
    int64_t amount = principal;
    int64_t y = 0;
    while (y < years) {
        amount += amount * rate_bp / 10000;
        y++;
    }
    return amount;
}

int main(void) {
    printf("add_tax=%lld\n", (long long)add_tax(1000, 825));
    printf("apply_discount=%lld\n", (long long)apply_discount(1000, 15));
    printf("tip_amount=%lld\n", (long long)tip_amount(4200, 18));
    printf("split_bill=%lld\n", (long long)split_bill(10000, 3));
    printf("split_remainder=%lld\n", (long long)split_remainder(10000, 3));
    printf("simple_interest=%lld\n", (long long)simple_interest(100000, 500, 3));
    printf("compound_interest=%lld\n", (long long)compound_interest(100000, 500, 3));
    printf("monthly_payment_flat=%lld\n", (long long)monthly_payment_flat(120000, 12));
    printf("cents_to_dollars=%lld\n", (long long)cents_to_dollars(12345));
    printf("cents_part=%lld\n", (long long)cents_part(12345));
    printf("percent_of=%lld\n", (long long)percent_of(45, 200));
    printf("markup=%lld\n", (long long)markup(5000, 40));
    printf("margin_pct=%lld\n", (long long)margin_pct(6000, 10000));
    printf("round_to_nearest=%lld\n", (long long)round_to_nearest(1237, 25));
    printf("future_value_years=%lld\n", (long long)future_value_years(100000, 500, 3));
    return 0;
}
