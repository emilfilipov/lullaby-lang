// Cross-language finance suite (JavaScript). Money/business math in integer cents.

function add_tax(cents, rate_bp) {
  return cents + Math.trunc(cents * rate_bp / 10000);
}

function apply_discount(cents, pct) {
  return cents - Math.trunc(cents * pct / 100);
}

function tip_amount(cents, pct) {
  return Math.trunc(cents * pct / 100);
}

function split_bill(cents, people) {
  return Math.trunc(cents / people);
}

function split_remainder(cents, people) {
  return cents % people;
}

function simple_interest(principal, rate_bp, years) {
  return Math.trunc(principal * rate_bp * years / 10000);
}

function compound_interest(principal, rate_bp, years) {
  let amount = principal;
  for (let i = 0; i < years; i++) {
    amount = amount + Math.trunc(amount * rate_bp / 10000);
  }
  return amount;
}

function monthly_payment_flat(principal, months) {
  return Math.trunc(principal / months);
}

function cents_to_dollars(cents) {
  return Math.trunc(cents / 100);
}

function cents_part(cents) {
  return cents % 100;
}

function percent_of(part, whole) {
  if (whole === 0) {
    return 0;
  }
  return Math.trunc(part * 100 / whole);
}

function markup(cost, pct) {
  return cost + Math.trunc(cost * pct / 100);
}

function margin_pct(cost, price) {
  if (price === 0) {
    return 0;
  }
  return Math.trunc((price - cost) * 100 / price);
}

function round_to_nearest(cents, step) {
  return Math.trunc((cents + Math.trunc(step / 2)) / step) * step;
}

function future_value_years(principal, rate_bp, years) {
  let amount = principal;
  let y = 0;
  while (y < years) {
    amount += Math.trunc(amount * rate_bp / 10000);
    y += 1;
  }
  return amount;
}

function main() {
  console.log("add_tax=" + add_tax(1000, 825));
  console.log("apply_discount=" + apply_discount(1000, 15));
  console.log("tip_amount=" + tip_amount(4200, 18));
  console.log("split_bill=" + split_bill(10000, 3));
  console.log("split_remainder=" + split_remainder(10000, 3));
  console.log("simple_interest=" + simple_interest(100000, 500, 3));
  console.log("compound_interest=" + compound_interest(100000, 500, 3));
  console.log("monthly_payment_flat=" + monthly_payment_flat(120000, 12));
  console.log("cents_to_dollars=" + cents_to_dollars(12345));
  console.log("cents_part=" + cents_part(12345));
  console.log("percent_of=" + percent_of(45, 200));
  console.log("markup=" + markup(5000, 40));
  console.log("margin_pct=" + margin_pct(6000, 10000));
  console.log("round_to_nearest=" + round_to_nearest(1237, 25));
  console.log("future_value_years=" + future_value_years(100000, 500, 3));
}

main();
