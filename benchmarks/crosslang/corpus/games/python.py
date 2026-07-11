"""Cross-language games suite (Python). Real, useful game/scoring logic over
int values and int lists. Lists are read only; no function mutates the
caller's data."""


def dice_sum(rolls: list, n: int) -> int:
    total = 0
    for i in range(n):
        total += rolls[i]
    return total


def all_same(a: list, n: int) -> int:
    for i in range(1, n):
        if a[i] != a[0]:
            return 0
    return 1


def count_value(a: list, n: int, v: int) -> int:
    count = 0
    for i in range(n):
        if a[i] == v:
            count += 1
    return count


def is_straight(a: list, n: int) -> int:
    for i in range(1, n):
        if a[i] != a[i - 1] + 1:
            return 0
    return 1


def high_card(a: list, n: int) -> int:
    m = a[0]
    for i in range(1, n):
        if a[i] > m:
            m = a[i]
    return m


def pair_count(a: list, n: int) -> int:
    count = 0
    i = 0
    while i < n:
        j = i
        while j < n and a[j] == a[i]:
            j += 1
        count += (j - i) // 2
        i = j
    return count


def line_winner(b: list, i: int, j: int, k: int) -> int:
    if b[i] != 0 and b[i] == b[j] and b[i] == b[k]:
        return b[i]
    return 0


def tic_tac_toe_winner(board: list) -> int:
    lines = [
        (0, 1, 2), (3, 4, 5), (6, 7, 8),
        (0, 3, 6), (1, 4, 7), (2, 5, 8),
        (0, 4, 8), (2, 4, 6),
    ]
    for i, j, k in lines:
        w = line_winner(board, i, j, k)
        if w != 0:
            return w
    return 0


def damage_calc(atk: int, def_: int) -> int:
    d = atk - def_
    return 1 if d < 1 else d


def xp_for_level(lvl: int) -> int:
    total = 0
    for i in range(1, lvl + 1):
        total += i * 100
    return total


def level_from_xp(xp: int) -> int:
    level = 0
    while xp_for_level(level + 1) <= xp:
        level += 1
    return level


def blackjack_value(cards: list, n: int) -> int:
    total = 0
    aces = 0
    for i in range(n):
        c = cards[i]
        if c == 1:
            aces += 1
            total += 11
        elif c >= 10:
            total += 10
        else:
            total += c
    while total > 21 and aces > 0:
        total -= 10
        aces -= 1
    return total


def is_flush(suits: list, n: int) -> int:
    for i in range(1, n):
        if suits[i] != suits[0]:
            return 0
    return 1


def grid_move_valid(x: int, y: int, dx: int, dy: int, w: int, h: int) -> int:
    nx = x + dx
    ny = y + dy
    return 1 if (0 <= nx < w and 0 <= ny < h) else 0


def dice_score_multiplier(roll: int) -> int:
    if roll == 6:
        return 3
    if roll == 5:
        return 2
    return 1


def bowling_frame_score(a: int, b: int) -> int:
    return a + b


def snake_grow(length: int, food: int) -> int:
    return length + food


def combo_multiplier(streak: int) -> int:
    if streak >= 20:
        return 4
    if streak >= 10:
        return 3
    if streak >= 5:
        return 2
    return 1


def score_with_bonus(base: int, mult: int) -> int:
    return base * mult


def main() -> None:
    dice = [3, 3, 3, 3, 3]
    straight = [1, 2, 3, 4, 5]
    pairs = [1, 1, 2, 3, 3, 3]
    board = [1, 1, 1, 2, 2, 0, 0, 0, 0]
    hand = [1, 10]
    suits = [2, 2, 2, 2, 2]
    print("dice_sum=" + str(dice_sum(dice, 5)))
    print("all_same=" + str(all_same(dice, 5)))
    print("count_value=" + str(count_value(dice, 5, 3)))
    print("is_straight=" + str(is_straight(straight, 5)))
    print("high_card=" + str(high_card(straight, 5)))
    print("pair_count=" + str(pair_count(pairs, 6)))
    print("tic_tac_toe_winner=" + str(tic_tac_toe_winner(board)))
    print("damage_calc=" + str(damage_calc(10, 3)))
    print("xp_for_level=" + str(xp_for_level(5)))
    print("level_from_xp=" + str(level_from_xp(1500)))
    print("blackjack_value=" + str(blackjack_value(hand, 2)))
    print("is_flush=" + str(is_flush(suits, 5)))
    print("grid_move_valid=" + str(grid_move_valid(3, 3, 1, 0, 8, 8)))
    print("dice_score_multiplier=" + str(dice_score_multiplier(6)))
    print("bowling_frame_score=" + str(bowling_frame_score(4, 5)))
    print("snake_grow=" + str(snake_grow(3, 2)))
    print("combo_multiplier=" + str(combo_multiplier(12)))
    print("score_with_bonus=" + str(score_with_bonus(100, 3)))


if __name__ == "__main__":
    main()
