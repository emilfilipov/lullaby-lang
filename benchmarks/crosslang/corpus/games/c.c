/* Cross-language games suite (C). Real, useful game/scoring logic over i64
   values and i64 arrays. Arrays are passed as const pointer + length; no
   function mutates the caller's data. */
#include <stdio.h>
#include <stdint.h>

int64_t dice_sum(const int64_t *rolls, int64_t n) {
    int64_t sum = 0;
    for (int64_t i = 0; i < n; i++) sum += rolls[i];
    return sum;
}

int64_t all_same(const int64_t *a, int64_t n) {
    for (int64_t i = 1; i < n; i++) if (a[i] != a[0]) return 0;
    return 1;
}

int64_t count_value(const int64_t *a, int64_t n, int64_t v) {
    int64_t count = 0;
    for (int64_t i = 0; i < n; i++) if (a[i] == v) count++;
    return count;
}

int64_t is_straight(const int64_t *a, int64_t n) {
    for (int64_t i = 1; i < n; i++) if (a[i] != a[i - 1] + 1) return 0;
    return 1;
}

int64_t high_card(const int64_t *a, int64_t n) {
    int64_t m = a[0];
    for (int64_t i = 1; i < n; i++) if (a[i] > m) m = a[i];
    return m;
}

int64_t pair_count(const int64_t *a, int64_t n) {
    int64_t count = 0, i = 0;
    while (i < n) {
        int64_t j = i;
        while (j < n && a[j] == a[i]) j++;
        count += (j - i) / 2;
        i = j;
    }
    return count;
}

int64_t line_winner(const int64_t *b, int64_t i, int64_t j, int64_t k) {
    if (b[i] != 0 && b[i] == b[j] && b[i] == b[k]) return b[i];
    return 0;
}

int64_t tic_tac_toe_winner(const int64_t *board) {
    int64_t w;
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

int64_t damage_calc(int64_t atk, int64_t def) {
    int64_t d = atk - def;
    return d < 1 ? 1 : d;
}

int64_t xp_for_level(int64_t lvl) {
    int64_t total = 0;
    for (int64_t i = 1; i <= lvl; i++) total += i * 100;
    return total;
}

int64_t level_from_xp(int64_t xp) {
    int64_t level = 0;
    while (xp_for_level(level + 1) <= xp) level++;
    return level;
}

int64_t blackjack_value(const int64_t *cards, int64_t n) {
    int64_t sum = 0, aces = 0;
    for (int64_t i = 0; i < n; i++) {
        int64_t c = cards[i];
        if (c == 1) { aces++; sum += 11; }
        else if (c >= 10) sum += 10;
        else sum += c;
    }
    while (sum > 21 && aces > 0) { sum -= 10; aces--; }
    return sum;
}

int64_t is_flush(const int64_t *suits, int64_t n) {
    for (int64_t i = 1; i < n; i++) if (suits[i] != suits[0]) return 0;
    return 1;
}

int64_t grid_move_valid(int64_t x, int64_t y, int64_t dx, int64_t dy, int64_t w, int64_t h) {
    int64_t nx = x + dx, ny = y + dy;
    return (nx >= 0 && nx < w && ny >= 0 && ny < h) ? 1 : 0;
}

int64_t dice_score_multiplier(int64_t roll) {
    if (roll == 6) return 3;
    if (roll == 5) return 2;
    return 1;
}

int64_t bowling_frame_score(int64_t a, int64_t b) {
    return a + b;
}

int64_t snake_grow(int64_t len, int64_t food) {
    return len + food;
}

int64_t combo_multiplier(int64_t streak) {
    if (streak >= 20) return 4;
    if (streak >= 10) return 3;
    if (streak >= 5) return 2;
    return 1;
}

int64_t score_with_bonus(int64_t base, int64_t mult) {
    return base * mult;
}

int main(void) {
    int64_t dice[5] = { 3, 3, 3, 3, 3 };
    int64_t straight[5] = { 1, 2, 3, 4, 5 };
    int64_t pairs[6] = { 1, 1, 2, 3, 3, 3 };
    int64_t board[9] = { 1, 1, 1, 2, 2, 0, 0, 0, 0 };
    int64_t hand[2] = { 1, 10 };
    int64_t suits[5] = { 2, 2, 2, 2, 2 };
    printf("dice_sum=%lld\n", (long long)dice_sum(dice, 5));
    printf("all_same=%lld\n", (long long)all_same(dice, 5));
    printf("count_value=%lld\n", (long long)count_value(dice, 5, 3));
    printf("is_straight=%lld\n", (long long)is_straight(straight, 5));
    printf("high_card=%lld\n", (long long)high_card(straight, 5));
    printf("pair_count=%lld\n", (long long)pair_count(pairs, 6));
    printf("tic_tac_toe_winner=%lld\n", (long long)tic_tac_toe_winner(board));
    printf("damage_calc=%lld\n", (long long)damage_calc(10, 3));
    printf("xp_for_level=%lld\n", (long long)xp_for_level(5));
    printf("level_from_xp=%lld\n", (long long)level_from_xp(1500));
    printf("blackjack_value=%lld\n", (long long)blackjack_value(hand, 2));
    printf("is_flush=%lld\n", (long long)is_flush(suits, 5));
    printf("grid_move_valid=%lld\n", (long long)grid_move_valid(3, 3, 1, 0, 8, 8));
    printf("dice_score_multiplier=%lld\n", (long long)dice_score_multiplier(6));
    printf("bowling_frame_score=%lld\n", (long long)bowling_frame_score(4, 5));
    printf("snake_grow=%lld\n", (long long)snake_grow(3, 2));
    printf("combo_multiplier=%lld\n", (long long)combo_multiplier(12));
    printf("score_with_bonus=%lld\n", (long long)score_with_bonus(100, 3));
    return 0;
}
