// Cross-language business suite (C++). Inventory/pricing/operations in integer cents/units.
#include <cstdint>
#include <iostream>

int64_t reorder_needed(int64_t stock, int64_t threshold) {
    return stock <= threshold ? 1 : 0;
}

int64_t stock_after(int64_t stock, int64_t sold, int64_t received) {
    return stock - sold + received;
}

int64_t bulk_price(int64_t qty, int64_t unit_cents) {
    int64_t base = qty * unit_cents;
    if (qty >= 100) return base - base * 10 / 100;
    return base;
}

int64_t shipping_cost(int64_t weight_g) {
    if (weight_g <= 500) return 500;
    if (weight_g <= 2000) return 1000;
    if (weight_g <= 10000) return 2500;
    return 5000;
}

int64_t tax_bracket(int64_t income) {
    int64_t tax = 0;
    if (income > 8500000) {
        tax += (income - 8500000) * 30 / 100;
        income = 8500000;
    }
    if (income > 4000000) {
        tax += (income - 4000000) * 20 / 100;
        income = 4000000;
    }
    if (income > 1000000) {
        tax += (income - 1000000) * 10 / 100;
    }
    return tax;
}

int64_t sales_commission(int64_t sales, int64_t rate_bp) {
    return sales * rate_bp / 10000;
}

int64_t loyalty_points(int64_t spend_cents) {
    return spend_cents / 100;
}

int64_t discount_tier(int64_t qty) {
    if (qty >= 100) return 20;
    if (qty >= 50) return 10;
    if (qty >= 10) return 5;
    return 0;
}

int64_t unit_price(int64_t total_cents, int64_t qty) {
    if (qty == 0) return 0;
    return total_cents / qty;
}

int64_t break_even_units(int64_t fixed, int64_t price, int64_t var) {
    if (price <= var) return 0;
    return fixed / (price - var);
}

int64_t gross_margin_pct(int64_t cost, int64_t price) {
    if (price == 0) return 0;
    return (price - cost) * 100 / price;
}

int64_t net_after_fees(int64_t gross, int64_t fee_bp) {
    return gross - gross * fee_bp / 10000;
}

int64_t invoice_total(int64_t subtotal, int64_t tax_bp, int64_t ship) {
    return subtotal + subtotal * tax_bp / 10000 + ship;
}

int64_t late_fee(int64_t days, int64_t amount) {
    if (days <= 0) return 0;
    return amount * 5 / 100 + amount * days / 100;
}

int64_t restock_quantity(int64_t current, int64_t max, int64_t min) {
    if (current <= min) return max - current;
    return 0;
}

int64_t profit(int64_t revenue, int64_t cost) {
    return revenue - cost;
}

int64_t roi_pct(int64_t gain, int64_t cost) {
    if (cost == 0) return 0;
    return gain * 100 / cost;
}

int64_t markup_price(int64_t cost, int64_t pct) {
    return cost + cost * pct / 100;
}

int main() {
    std::cout << "reorder_needed=" << reorder_needed(8, 10) << "\n";
    std::cout << "stock_after=" << stock_after(100, 30, 50) << "\n";
    std::cout << "bulk_price=" << bulk_price(120, 500) << "\n";
    std::cout << "shipping_cost=" << shipping_cost(1500) << "\n";
    std::cout << "tax_bracket=" << tax_bracket(5000000) << "\n";
    std::cout << "sales_commission=" << sales_commission(1000000, 250) << "\n";
    std::cout << "loyalty_points=" << loyalty_points(45678) << "\n";
    std::cout << "discount_tier=" << discount_tier(60) << "\n";
    std::cout << "unit_price=" << unit_price(10000, 8) << "\n";
    std::cout << "break_even_units=" << break_even_units(500000, 1500, 500) << "\n";
    std::cout << "gross_margin_pct=" << gross_margin_pct(6000, 10000) << "\n";
    std::cout << "net_after_fees=" << net_after_fees(100000, 290) << "\n";
    std::cout << "invoice_total=" << invoice_total(50000, 825, 1000) << "\n";
    std::cout << "late_fee=" << late_fee(5, 20000) << "\n";
    std::cout << "restock_quantity=" << restock_quantity(5, 100, 10) << "\n";
    std::cout << "profit=" << profit(120000, 90000) << "\n";
    std::cout << "roi_pct=" << roi_pct(30000, 90000) << "\n";
    std::cout << "markup_price=" << markup_price(5000, 40) << "\n";
    return 0;
}
