// Cross-language business suite (JavaScript). Inventory/pricing/operations in integer cents/units.

function reorder_needed(stock, threshold) {
    return stock <= threshold ? 1 : 0;
}

function stock_after(stock, sold, received) {
    return stock - sold + received;
}

function bulk_price(qty, unit_cents) {
    const base = qty * unit_cents;
    if (qty >= 100) return base - Math.trunc(base * 10 / 100);
    return base;
}

function shipping_cost(weight_g) {
    if (weight_g <= 500) return 500;
    if (weight_g <= 2000) return 1000;
    if (weight_g <= 10000) return 2500;
    return 5000;
}

function tax_bracket(income) {
    let tax = 0;
    if (income > 8500000) {
        tax += Math.trunc((income - 8500000) * 30 / 100);
        income = 8500000;
    }
    if (income > 4000000) {
        tax += Math.trunc((income - 4000000) * 20 / 100);
        income = 4000000;
    }
    if (income > 1000000) {
        tax += Math.trunc((income - 1000000) * 10 / 100);
    }
    return tax;
}

function sales_commission(sales, rate_bp) {
    return Math.trunc(sales * rate_bp / 10000);
}

function loyalty_points(spend_cents) {
    return Math.trunc(spend_cents / 100);
}

function discount_tier(qty) {
    if (qty >= 100) return 20;
    if (qty >= 50) return 10;
    if (qty >= 10) return 5;
    return 0;
}

function unit_price(total_cents, qty) {
    if (qty === 0) return 0;
    return Math.trunc(total_cents / qty);
}

function break_even_units(fixed, price, var_cost) {
    if (price <= var_cost) return 0;
    return Math.trunc(fixed / (price - var_cost));
}

function gross_margin_pct(cost, price) {
    if (price === 0) return 0;
    return Math.trunc((price - cost) * 100 / price);
}

function net_after_fees(gross, fee_bp) {
    return gross - Math.trunc(gross * fee_bp / 10000);
}

function invoice_total(subtotal, tax_bp, ship) {
    return subtotal + Math.trunc(subtotal * tax_bp / 10000) + ship;
}

function late_fee(days, amount) {
    if (days <= 0) return 0;
    return Math.trunc(amount * 5 / 100) + Math.trunc(amount * days / 100);
}

function restock_quantity(current, max, min) {
    if (current <= min) return max - current;
    return 0;
}

function profit(revenue, cost) {
    return revenue - cost;
}

function roi_pct(gain, cost) {
    if (cost === 0) return 0;
    return Math.trunc(gain * 100 / cost);
}

function markup_price(cost, pct) {
    return cost + Math.trunc(cost * pct / 100);
}

function main() {
    console.log("reorder_needed=" + reorder_needed(8, 10));
    console.log("stock_after=" + stock_after(100, 30, 50));
    console.log("bulk_price=" + bulk_price(120, 500));
    console.log("shipping_cost=" + shipping_cost(1500));
    console.log("tax_bracket=" + tax_bracket(5000000));
    console.log("sales_commission=" + sales_commission(1000000, 250));
    console.log("loyalty_points=" + loyalty_points(45678));
    console.log("discount_tier=" + discount_tier(60));
    console.log("unit_price=" + unit_price(10000, 8));
    console.log("break_even_units=" + break_even_units(500000, 1500, 500));
    console.log("gross_margin_pct=" + gross_margin_pct(6000, 10000));
    console.log("net_after_fees=" + net_after_fees(100000, 290));
    console.log("invoice_total=" + invoice_total(50000, 825, 1000));
    console.log("late_fee=" + late_fee(5, 20000));
    console.log("restock_quantity=" + restock_quantity(5, 100, 10));
    console.log("profit=" + profit(120000, 90000));
    console.log("roi_pct=" + roi_pct(30000, 90000));
    console.log("markup_price=" + markup_price(5000, 40));
}

main();
