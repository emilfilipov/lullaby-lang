// Cross-language games suite (Rust). Real, useful game/scoring logic over i64
// values and i64 slices. Slices are borrowed read-only; no function mutates
// the caller's data.

fn dice_sum(rolls: &[i64], n: i64) -> i64 {
    let mut sum = 0;
    for i in 0..n as usize { sum += rolls[i]; }
    sum
}

fn all_same(a: &[i64], n: i64) -> i64 {
    for i in 1..n as usize { if a[i] != a[0] { return 0; } }
    1
}

fn count_value(a: &[i64], n: i64, v: i64) -> i64 {
    let mut count = 0;
    for i in 0..n as usize { if a[i] == v { count += 1; } }
    count
}

fn is_straight(a: &[i64], n: i64) -> i64 {
    for i in 1..n as usize { if a[i] != a[i - 1] + 1 { return 0; } }
    1
}

fn high_card(a: &[i64], n: i64) -> i64 {
    let mut m = a[0];
    for i in 1..n as usize { if a[i] > m { m = a[i]; } }
    m
}

fn pair_count(a: &[i64], n: i64) -> i64 {
    let mut count = 0;
    let mut i = 0usize;
    let n = n as usize;
    while i < n {
        let mut j = i;
        while j < n && a[j] == a[i] { j += 1; }
        count += (j - i) as i64 / 2;
        i = j;
    }
    count
}

fn line_winner(b: &[i64], i: usize, j: usize, k: usize) -> i64 {
    if b[i] != 0 && b[i] == b[j] && b[i] == b[k] { return b[i]; }
    0
}

fn tic_tac_toe_winner(board: &[i64]) -> i64 {
    let mut w;
    w = line_winner(board, 0, 1, 2); if w != 0 { return w; }
    w = line_winner(board, 3, 4, 5); if w != 0 { return w; }
    w = line_winner(board, 6, 7, 8); if w != 0 { return w; }
    w = line_winner(board, 0, 3, 6); if w != 0 { return w; }
    w = line_winner(board, 1, 4, 7); if w != 0 { return w; }
    w = line_winner(board, 2, 5, 8); if w != 0 { return w; }
    w = line_winner(board, 0, 4, 8); if w != 0 { return w; }
    w = line_winner(board, 2, 4, 6); if w != 0 { return w; }
    0
}

fn damage_calc(atk: i64, def: i64) -> i64 {
    let d = atk - def;
    if d < 1 { 1 } else { d }
}

fn xp_for_level(lvl: i64) -> i64 {
    let mut total = 0;
    for i in 1..=lvl { total += i * 100; }
    total
}

fn level_from_xp(xp: i64) -> i64 {
    let mut level = 0;
    while xp_for_level(level + 1) <= xp { level += 1; }
    level
}

fn blackjack_value(cards: &[i64], n: i64) -> i64 {
    let mut sum = 0;
    let mut aces = 0;
    for i in 0..n as usize {
        let c = cards[i];
        if c == 1 { aces += 1; sum += 11; }
        else if c >= 10 { sum += 10; }
        else { sum += c; }
    }
    while sum > 21 && aces > 0 { sum -= 10; aces -= 1; }
    sum
}

fn is_flush(suits: &[i64], n: i64) -> i64 {
    for i in 1..n as usize { if suits[i] != suits[0] { return 0; } }
    1
}

fn grid_move_valid(x: i64, y: i64, dx: i64, dy: i64, w: i64, h: i64) -> i64 {
    let nx = x + dx;
    let ny = y + dy;
    if nx >= 0 && nx < w && ny >= 0 && ny < h { 1 } else { 0 }
}

fn dice_score_multiplier(roll: i64) -> i64 {
    if roll == 6 { return 3; }
    if roll == 5 { return 2; }
    1
}

fn bowling_frame_score(a: i64, b: i64) -> i64 {
    a + b
}

fn snake_grow(len: i64, food: i64) -> i64 {
    len + food
}

fn combo_multiplier(streak: i64) -> i64 {
    if streak >= 20 { return 4; }
    if streak >= 10 { return 3; }
    if streak >= 5 { return 2; }
    1
}

fn score_with_bonus(base: i64, mult: i64) -> i64 {
    base * mult
}

fn main() {
    let dice = [3i64, 3, 3, 3, 3];
    let straight = [1i64, 2, 3, 4, 5];
    let pairs = [1i64, 1, 2, 3, 3, 3];
    let board = [1i64, 1, 1, 2, 2, 0, 0, 0, 0];
    let hand = [1i64, 10];
    let suits = [2i64, 2, 2, 2, 2];
    println!("dice_sum={}", dice_sum(&dice, 5));
    println!("all_same={}", all_same(&dice, 5));
    println!("count_value={}", count_value(&dice, 5, 3));
    println!("is_straight={}", is_straight(&straight, 5));
    println!("high_card={}", high_card(&straight, 5));
    println!("pair_count={}", pair_count(&pairs, 6));
    println!("tic_tac_toe_winner={}", tic_tac_toe_winner(&board));
    println!("damage_calc={}", damage_calc(10, 3));
    println!("xp_for_level={}", xp_for_level(5));
    println!("level_from_xp={}", level_from_xp(1500));
    println!("blackjack_value={}", blackjack_value(&hand, 2));
    println!("is_flush={}", is_flush(&suits, 5));
    println!("grid_move_valid={}", grid_move_valid(3, 3, 1, 0, 8, 8));
    println!("dice_score_multiplier={}", dice_score_multiplier(6));
    println!("bowling_frame_score={}", bowling_frame_score(4, 5));
    println!("snake_grow={}", snake_grow(3, 2));
    println!("combo_multiplier={}", combo_multiplier(12));
    println!("score_with_bonus={}", score_with_bonus(100, 3));
}
