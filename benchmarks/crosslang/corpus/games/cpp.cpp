// Cross-language games suite (C++). Real, useful game/scoring logic over i64
// values and i64 arrays. Arrays are passed as const pointer + length; no
// function mutates the caller's data.
#include <cstdint>
#include <iostream>

std::int64_t dice_sum(const std::int64_t *rolls, std::int64_t n) {
    std::int64_t sum = 0;
    for (std::int64_t i = 0; i < n; i++) sum += rolls[i];
    return sum;
}

std::int64_t all_same(const std::int64_t *a, std::int64_t n) {
    for (std::int64_t i = 1; i < n; i++) if (a[i] != a[0]) return 0;
    return 1;
}

std::int64_t count_value(const std::int64_t *a, std::int64_t n, std::int64_t v) {
    std::int64_t count = 0;
    for (std::int64_t i = 0; i < n; i++) if (a[i] == v) count++;
    return count;
}

std::int64_t is_straight(const std::int64_t *a, std::int64_t n) {
    for (std::int64_t i = 1; i < n; i++) if (a[i] != a[i - 1] + 1) return 0;
    return 1;
}

std::int64_t high_card(const std::int64_t *a, std::int64_t n) {
    std::int64_t m = a[0];
    for (std::int64_t i = 1; i < n; i++) if (a[i] > m) m = a[i];
    return m;
}

std::int64_t pair_count(const std::int64_t *a, std::int64_t n) {
    std::int64_t count = 0, i = 0;
    while (i < n) {
        std::int64_t j = i;
        while (j < n && a[j] == a[i]) j++;
        count += (j - i) / 2;
        i = j;
    }
    return count;
}

std::int64_t line_winner(const std::int64_t *b, std::int64_t i, std::int64_t j, std::int64_t k) {
    if (b[i] != 0 && b[i] == b[j] && b[i] == b[k]) return b[i];
    return 0;
}

std::int64_t tic_tac_toe_winner(const std::int64_t *board) {
    std::int64_t w;
    if ((w = line_winner(board, 0, 1, 2))) return w;
    if ((w = line_winner(board, 3, 4, 5))) return w;
    if ((w = line_winner(board, 6, 7, 8))) return w;
    if ((w = line_winner(board, 0, 3, 6))) return w;
    if ((w = line_winner(board, 1, 4, 7))) return w;
    if ((w = line_winner(board, 2, 5, 8))) return w;
    if ((w = line_winner(board, 0, 4, 8))) return w;
    if ((w = line_winner(board, 2, 4, 6))) return w;
    return 0;
}

std::int64_t damage_calc(std::int64_t atk, std::int64_t def) {
    std::int64_t d = atk - def;
    return d < 1 ? 1 : d;
}

std::int64_t xp_for_level(std::int64_t lvl) {
    std::int64_t total = 0;
    for (std::int64_t i = 1; i <= lvl; i++) total += i * 100;
    return total;
}

std::int64_t level_from_xp(std::int64_t xp) {
    std::int64_t level = 0;
    while (xp_for_level(level + 1) <= xp) level++;
    return level;
}

std::int64_t blackjack_value(const std::int64_t *cards, std::int64_t n) {
    std::int64_t sum = 0, aces = 0;
    for (std::int64_t i = 0; i < n; i++) {
        std::int64_t c = cards[i];
        if (c == 1) { aces++; sum += 11; }
        else if (c >= 10) sum += 10;
        else sum += c;
    }
    while (sum > 21 && aces > 0) { sum -= 10; aces--; }
    return sum;
}

std::int64_t is_flush(const std::int64_t *suits, std::int64_t n) {
    for (std::int64_t i = 1; i < n; i++) if (suits[i] != suits[0]) return 0;
    return 1;
}

std::int64_t grid_move_valid(std::int64_t x, std::int64_t y, std::int64_t dx, std::int64_t dy, std::int64_t w, std::int64_t h) {
    std::int64_t nx = x + dx, ny = y + dy;
    return (nx >= 0 && nx < w && ny >= 0 && ny < h) ? 1 : 0;
}

std::int64_t dice_score_multiplier(std::int64_t roll) {
    if (roll == 6) return 3;
    if (roll == 5) return 2;
    return 1;
}

std::int64_t bowling_frame_score(std::int64_t a, std::int64_t b) {
    return a + b;
}

std::int64_t snake_grow(std::int64_t len, std::int64_t food) {
    return len + food;
}

std::int64_t combo_multiplier(std::int64_t streak) {
    if (streak >= 20) return 4;
    if (streak >= 10) return 3;
    if (streak >= 5) return 2;
    return 1;
}

std::int64_t score_with_bonus(std::int64_t base, std::int64_t mult) {
    return base * mult;
}

int main() {
    std::int64_t dice[5] = { 3, 3, 3, 3, 3 };
    std::int64_t straight[5] = { 1, 2, 3, 4, 5 };
    std::int64_t pairs[6] = { 1, 1, 2, 3, 3, 3 };
    std::int64_t board[9] = { 1, 1, 1, 2, 2, 0, 0, 0, 0 };
    std::int64_t hand[2] = { 1, 10 };
    std::int64_t suits[5] = { 2, 2, 2, 2, 2 };
    std::cout << "dice_sum=" << dice_sum(dice, 5) << "\n";
    std::cout << "all_same=" << all_same(dice, 5) << "\n";
    std::cout << "count_value=" << count_value(dice, 5, 3) << "\n";
    std::cout << "is_straight=" << is_straight(straight, 5) << "\n";
    std::cout << "high_card=" << high_card(straight, 5) << "\n";
    std::cout << "pair_count=" << pair_count(pairs, 6) << "\n";
    std::cout << "tic_tac_toe_winner=" << tic_tac_toe_winner(board) << "\n";
    std::cout << "damage_calc=" << damage_calc(10, 3) << "\n";
    std::cout << "xp_for_level=" << xp_for_level(5) << "\n";
    std::cout << "level_from_xp=" << level_from_xp(1500) << "\n";
    std::cout << "blackjack_value=" << blackjack_value(hand, 2) << "\n";
    std::cout << "is_flush=" << is_flush(suits, 5) << "\n";
    std::cout << "grid_move_valid=" << grid_move_valid(3, 3, 1, 0, 8, 8) << "\n";
    std::cout << "dice_score_multiplier=" << dice_score_multiplier(6) << "\n";
    std::cout << "bowling_frame_score=" << bowling_frame_score(4, 5) << "\n";
    std::cout << "snake_grow=" << snake_grow(3, 2) << "\n";
    std::cout << "combo_multiplier=" << combo_multiplier(12) << "\n";
    std::cout << "score_with_bonus=" << score_with_bonus(100, 3) << "\n";
    return 0;
}
