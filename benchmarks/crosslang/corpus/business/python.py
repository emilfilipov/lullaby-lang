# Cross-language business suite (Python). Inventory/pricing/operations in integer cents/units.


def reorder_needed(stock, threshold):
    return 1 if stock <= threshold else 0


def stock_after(stock, sold, received):
    return stock - sold + received


def bulk_price(qty, unit_cents):
    base = qty * unit_cents
    if qty >= 100:
        return base - base * 10 // 100
    return base


def shipping_cost(weight_g):
    if weight_g <= 500:
        return 500
    if weight_g <= 2000:
        return 1000
    if weight_g <= 10000:
        return 2500
    return 5000


def tax_bracket(income):
    tax = 0
    if income > 8500000:
        tax += (income - 8500000) * 30 // 100
        income = 8500000
    if income > 4000000:
        tax += (income - 4000000) * 20 // 100
        income = 4000000
    if income > 1000000:
        tax += (income - 1000000) * 10 // 100
    return tax


def sales_commission(sales, rate_bp):
    return sales * rate_bp // 10000


def loyalty_points(spend_cents):
    return spend_cents // 100


def discount_tier(qty):
    if qty >= 100:
        return 20
    if qty >= 50:
        return 10
    if qty >= 10:
        return 5
    return 0


def unit_price(total_cents, qty):
    if qty == 0:
        return 0
    return total_cents // qty


def break_even_units(fixed, price, var):
    if price <= var:
        return 0
    return fixed // (price - var)


def gross_margin_pct(cost, price):
    if price == 0:
        return 0
    return (price - cost) * 100 // price


def net_after_fees(gross, fee_bp):
    return gross - gross * fee_bp // 10000


def invoice_total(subtotal, tax_bp, ship):
    return subtotal + subtotal * tax_bp // 10000 + ship


def late_fee(days, amount):
    if days <= 0:
        return 0
    return amount * 5 // 100 + amount * days // 100


def restock_quantity(current, max_level, min_level):
    if current <= min_level:
        return max_level - current
    return 0


def profit(revenue, cost):
    return revenue - cost


def roi_pct(gain, cost):
    if cost == 0:
        return 0
    return gain * 100 // cost


def markup_price(cost, pct):
    return cost + cost * pct // 100


def main():
    print("reorder_needed=" + str(reorder_needed(8, 10)))
    print("stock_after=" + str(stock_after(100, 30, 50)))
    print("bulk_price=" + str(bulk_price(120, 500)))
    print("shipping_cost=" + str(shipping_cost(1500)))
    print("tax_bracket=" + str(tax_bracket(5000000)))
    print("sales_commission=" + str(sales_commission(1000000, 250)))
    print("loyalty_points=" + str(loyalty_points(45678)))
    print("discount_tier=" + str(discount_tier(60)))
    print("unit_price=" + str(unit_price(10000, 8)))
    print("break_even_units=" + str(break_even_units(500000, 1500, 500)))
    print("gross_margin_pct=" + str(gross_margin_pct(6000, 10000)))
    print("net_after_fees=" + str(net_after_fees(100000, 290)))
    print("invoice_total=" + str(invoice_total(50000, 825, 1000)))
    print("late_fee=" + str(late_fee(5, 20000)))
    print("restock_quantity=" + str(restock_quantity(5, 100, 10)))
    print("profit=" + str(profit(120000, 90000)))
    print("roi_pct=" + str(roi_pct(30000, 90000)))
    print("markup_price=" + str(markup_price(5000, 40)))


if __name__ == "__main__":
    main()
