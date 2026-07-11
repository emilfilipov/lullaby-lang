// Cross-language games suite (JavaScript). Real, useful game/scoring logic over
// int values and int arrays. Arrays are read only; no function mutates the
// caller's data.

function dice_sum(rolls, n) {
  let total = 0;
  for (let i = 0; i < n; i++) {
    total += rolls[i];
  }
  return total;
}

function all_same(a, n) {
  for (let i = 1; i < n; i++) {
    if (a[i] !== a[0]) {
      return 0;
    }
  }
  return 1;
}

function count_value(a, n, v) {
  let count = 0;
  for (let i = 0; i < n; i++) {
    if (a[i] === v) {
      count += 1;
    }
  }
  return count;
}

function is_straight(a, n) {
  for (let i = 1; i < n; i++) {
    if (a[i] !== a[i - 1] + 1) {
      return 0;
    }
  }
  return 1;
}

function high_card(a, n) {
  let m = a[0];
  for (let i = 1; i < n; i++) {
    if (a[i] > m) {
      m = a[i];
    }
  }
  return m;
}

function pair_count(a, n) {
  let count = 0;
  let i = 0;
  while (i < n) {
    let j = i;
    while (j < n && a[j] === a[i]) {
      j += 1;
    }
    count += Math.trunc((j - i) / 2);
    i = j;
  }
  return count;
}

function line_winner(b, i, j, k) {
  if (b[i] !== 0 && b[i] === b[j] && b[i] === b[k]) {
    return b[i];
  }
  return 0;
}

function tic_tac_toe_winner(board) {
  const lines = [
    [0, 1, 2], [3, 4, 5], [6, 7, 8],
    [0, 3, 6], [1, 4, 7], [2, 5, 8],
    [0, 4, 8], [2, 4, 6],
  ];
  for (let l = 0; l < lines.length; l++) {
    const w = line_winner(board, lines[l][0], lines[l][1], lines[l][2]);
    if (w !== 0) {
      return w;
    }
  }
  return 0;
}

function damage_calc(atk, def) {
  const d = atk - def;
  return d < 1 ? 1 : d;
}

function xp_for_level(lvl) {
  let total = 0;
  for (let i = 1; i <= lvl; i++) {
    total += i * 100;
  }
  return total;
}

function level_from_xp(xp) {
  let level = 0;
  while (xp_for_level(level + 1) <= xp) {
    level += 1;
  }
  return level;
}

function blackjack_value(cards, n) {
  let total = 0;
  let aces = 0;
  for (let i = 0; i < n; i++) {
    const c = cards[i];
    if (c === 1) {
      aces += 1;
      total += 11;
    } else if (c >= 10) {
      total += 10;
    } else {
      total += c;
    }
  }
  while (total > 21 && aces > 0) {
    total -= 10;
    aces -= 1;
  }
  return total;
}

function is_flush(suits, n) {
  for (let i = 1; i < n; i++) {
    if (suits[i] !== suits[0]) {
      return 0;
    }
  }
  return 1;
}

function grid_move_valid(x, y, dx, dy, w, h) {
  const nx = x + dx;
  const ny = y + dy;
  return nx >= 0 && nx < w && ny >= 0 && ny < h ? 1 : 0;
}

function dice_score_multiplier(roll) {
  if (roll === 6) {
    return 3;
  }
  if (roll === 5) {
    return 2;
  }
  return 1;
}

function bowling_frame_score(a, b) {
  return a + b;
}

function snake_grow(length, food) {
  return length + food;
}

function combo_multiplier(streak) {
  if (streak >= 20) {
    return 4;
  }
  if (streak >= 10) {
    return 3;
  }
  if (streak >= 5) {
    return 2;
  }
  return 1;
}

function score_with_bonus(base, mult) {
  return base * mult;
}

function main() {
  const dice = [3, 3, 3, 3, 3];
  const straight = [1, 2, 3, 4, 5];
  const pairs = [1, 1, 2, 3, 3, 3];
  const board = [1, 1, 1, 2, 2, 0, 0, 0, 0];
  const hand = [1, 10];
  const suits = [2, 2, 2, 2, 2];
  console.log("dice_sum=" + dice_sum(dice, 5));
  console.log("all_same=" + all_same(dice, 5));
  console.log("count_value=" + count_value(dice, 5, 3));
  console.log("is_straight=" + is_straight(straight, 5));
  console.log("high_card=" + high_card(straight, 5));
  console.log("pair_count=" + pair_count(pairs, 6));
  console.log("tic_tac_toe_winner=" + tic_tac_toe_winner(board));
  console.log("damage_calc=" + damage_calc(10, 3));
  console.log("xp_for_level=" + xp_for_level(5));
  console.log("level_from_xp=" + level_from_xp(1500));
  console.log("blackjack_value=" + blackjack_value(hand, 2));
  console.log("is_flush=" + is_flush(suits, 5));
  console.log("grid_move_valid=" + grid_move_valid(3, 3, 1, 0, 8, 8));
  console.log("dice_score_multiplier=" + dice_score_multiplier(6));
  console.log("bowling_frame_score=" + bowling_frame_score(4, 5));
  console.log("snake_grow=" + snake_grow(3, 2));
  console.log("combo_multiplier=" + combo_multiplier(12));
  console.log("score_with_bonus=" + score_with_bonus(100, 3));
}

main();
