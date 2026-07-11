# Cross-language state_machines suite (Python). IntEnums + match-driven transitions.
from enum import IntEnum


class TrafficLight(IntEnum):
    Red = 0
    Green = 1
    Yellow = 2


class Direction(IntEnum):
    North = 0
    East = 1
    South = 2
    West = 3


class OrderStatus(IntEnum):
    Pending = 0
    Paid = 1
    Shipped = 2
    Delivered = 3
    Cancelled = 4


class Token(IntEnum):
    Num = 0
    Plus = 1
    Minus = 2
    Star = 3
    Slash = 4
    LParen = 5
    RParen = 6
    End = 7


def light_next(t):
    match t:
        case TrafficLight.Red:
            return TrafficLight.Green
        case TrafficLight.Green:
            return TrafficLight.Yellow
        case TrafficLight.Yellow:
            return TrafficLight.Red


def light_go(t):
    return 1 if t == TrafficLight.Green else 0


def light_duration(t):
    match t:
        case TrafficLight.Red:
            return 30
        case TrafficLight.Green:
            return 25
        case TrafficLight.Yellow:
            return 5


def turn_right(d):
    match d:
        case Direction.North:
            return Direction.East
        case Direction.East:
            return Direction.South
        case Direction.South:
            return Direction.West
        case Direction.West:
            return Direction.North


def turn_left(d):
    match d:
        case Direction.North:
            return Direction.West
        case Direction.West:
            return Direction.South
        case Direction.South:
            return Direction.East
        case Direction.East:
            return Direction.North


def opposite(d):
    match d:
        case Direction.North:
            return Direction.South
        case Direction.South:
            return Direction.North
        case Direction.East:
            return Direction.West
        case Direction.West:
            return Direction.East


def dir_dx(d):
    match d:
        case Direction.East:
            return 1
        case Direction.West:
            return -1
        case _:
            return 0


def dir_dy(d):
    match d:
        case Direction.North:
            return 1
        case Direction.South:
            return -1
        case _:
            return 0


def dir_clockwise_index(d):
    match d:
        case Direction.North:
            return 0
        case Direction.East:
            return 1
        case Direction.South:
            return 2
        case Direction.West:
            return 3


def order_can_cancel(s):
    return 1 if s in (OrderStatus.Pending, OrderStatus.Paid) else 0


def order_next(s):
    match s:
        case OrderStatus.Pending:
            return OrderStatus.Paid
        case OrderStatus.Paid:
            return OrderStatus.Shipped
        case OrderStatus.Shipped:
            return OrderStatus.Delivered
        case OrderStatus.Delivered:
            return OrderStatus.Delivered
        case OrderStatus.Cancelled:
            return OrderStatus.Cancelled


def order_is_final(s):
    return 1 if s in (OrderStatus.Delivered, OrderStatus.Cancelled) else 0


def token_precedence(t):
    match t:
        case Token.Plus | Token.Minus:
            return 1
        case Token.Star | Token.Slash:
            return 2
        case _:
            return 0


def token_is_operator(t):
    match t:
        case Token.Plus | Token.Minus | Token.Star | Token.Slash:
            return 1
        case _:
            return 0


def token_is_paren(t):
    return 1 if t in (Token.LParen, Token.RParen) else 0


def light_name(t):
    match t:
        case TrafficLight.Red:
            return "RED"
        case TrafficLight.Green:
            return "GREEN"
        case TrafficLight.Yellow:
            return "YELLOW"


def main():
    print("light_next=" + light_name(light_next(TrafficLight.Red)))
    print("light_go=" + str(light_go(TrafficLight.Green)))
    print("light_duration=" + str(light_duration(TrafficLight.Red)))
    print("turn_right(N)=clockwise " + str(dir_clockwise_index(turn_right(Direction.North))))
    print("turn_left(N)=clockwise " + str(dir_clockwise_index(turn_left(Direction.North))))
    print("opposite(N)=clockwise " + str(dir_clockwise_index(opposite(Direction.North))))
    print("dir_dx(East)=" + str(dir_dx(Direction.East)))
    print("dir_dy(South)=" + str(dir_dy(Direction.South)))
    print("order_can_cancel(Paid)=" + str(order_can_cancel(OrderStatus.Paid)))
    print("order_is_final(order_next(Shipped))=" + str(order_is_final(order_next(OrderStatus.Shipped))))
    print("token_precedence(Star)=" + str(token_precedence(Token.Star)))
    print("token_is_operator(Plus)=" + str(token_is_operator(Token.Plus)))
    print("token_is_paren(LParen)=" + str(token_is_paren(Token.LParen)))


if __name__ == "__main__":
    main()
