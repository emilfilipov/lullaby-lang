// Cross-language state_machines suite (JavaScript). Integer-constant enums + switch-driven transitions.

const TrafficLight = Object.freeze({ Red: 0, Green: 1, Yellow: 2 });
const Direction = Object.freeze({ North: 0, East: 1, South: 2, West: 3 });
const OrderStatus = Object.freeze({ Pending: 0, Paid: 1, Shipped: 2, Delivered: 3, Cancelled: 4 });
const Token = Object.freeze({ Num: 0, Plus: 1, Minus: 2, Star: 3, Slash: 4, LParen: 5, RParen: 6, End: 7 });

function light_next(t) {
  switch (t) {
    case TrafficLight.Red: return TrafficLight.Green;
    case TrafficLight.Green: return TrafficLight.Yellow;
    case TrafficLight.Yellow: return TrafficLight.Red;
  }
}

function light_go(t) {
  return t === TrafficLight.Green ? 1 : 0;
}

function light_duration(t) {
  switch (t) {
    case TrafficLight.Red: return 30;
    case TrafficLight.Green: return 25;
    case TrafficLight.Yellow: return 5;
  }
}

function turn_right(d) {
  switch (d) {
    case Direction.North: return Direction.East;
    case Direction.East: return Direction.South;
    case Direction.South: return Direction.West;
    case Direction.West: return Direction.North;
  }
}

function turn_left(d) {
  switch (d) {
    case Direction.North: return Direction.West;
    case Direction.West: return Direction.South;
    case Direction.South: return Direction.East;
    case Direction.East: return Direction.North;
  }
}

function opposite(d) {
  switch (d) {
    case Direction.North: return Direction.South;
    case Direction.South: return Direction.North;
    case Direction.East: return Direction.West;
    case Direction.West: return Direction.East;
  }
}

function dir_dx(d) {
  switch (d) {
    case Direction.East: return 1;
    case Direction.West: return -1;
    default: return 0;
  }
}

function dir_dy(d) {
  switch (d) {
    case Direction.North: return 1;
    case Direction.South: return -1;
    default: return 0;
  }
}

function dir_clockwise_index(d) {
  switch (d) {
    case Direction.North: return 0;
    case Direction.East: return 1;
    case Direction.South: return 2;
    case Direction.West: return 3;
  }
}

function order_can_cancel(s) {
  switch (s) {
    case OrderStatus.Pending:
    case OrderStatus.Paid: return 1;
    default: return 0;
  }
}

function order_next(s) {
  switch (s) {
    case OrderStatus.Pending: return OrderStatus.Paid;
    case OrderStatus.Paid: return OrderStatus.Shipped;
    case OrderStatus.Shipped: return OrderStatus.Delivered;
    case OrderStatus.Delivered: return OrderStatus.Delivered;
    case OrderStatus.Cancelled: return OrderStatus.Cancelled;
  }
}

function order_is_final(s) {
  switch (s) {
    case OrderStatus.Delivered:
    case OrderStatus.Cancelled: return 1;
    default: return 0;
  }
}

function token_precedence(t) {
  switch (t) {
    case Token.Plus:
    case Token.Minus: return 1;
    case Token.Star:
    case Token.Slash: return 2;
    default: return 0;
  }
}

function token_is_operator(t) {
  switch (t) {
    case Token.Plus:
    case Token.Minus:
    case Token.Star:
    case Token.Slash: return 1;
    default: return 0;
  }
}

function token_is_paren(t) {
  switch (t) {
    case Token.LParen:
    case Token.RParen: return 1;
    default: return 0;
  }
}

function light_name(t) {
  switch (t) {
    case TrafficLight.Red: return "RED";
    case TrafficLight.Green: return "GREEN";
    case TrafficLight.Yellow: return "YELLOW";
  }
}

function main() {
  console.log("light_next=" + light_name(light_next(TrafficLight.Red)));
  console.log("light_go=" + light_go(TrafficLight.Green));
  console.log("light_duration=" + light_duration(TrafficLight.Red));
  console.log("turn_right(N)=clockwise " + dir_clockwise_index(turn_right(Direction.North)));
  console.log("turn_left(N)=clockwise " + dir_clockwise_index(turn_left(Direction.North)));
  console.log("opposite(N)=clockwise " + dir_clockwise_index(opposite(Direction.North)));
  console.log("dir_dx(East)=" + dir_dx(Direction.East));
  console.log("dir_dy(South)=" + dir_dy(Direction.South));
  console.log("order_can_cancel(Paid)=" + order_can_cancel(OrderStatus.Paid));
  console.log("order_is_final(order_next(Shipped))=" + order_is_final(order_next(OrderStatus.Shipped)));
  console.log("token_precedence(Star)=" + token_precedence(Token.Star));
  console.log("token_is_operator(Plus)=" + token_is_operator(Token.Plus));
  console.log("token_is_paren(LParen)=" + token_is_paren(Token.LParen));
}

main();
