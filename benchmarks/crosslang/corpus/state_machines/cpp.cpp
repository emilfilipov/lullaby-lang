// Cross-language state_machines suite (C++). Scoped enums + switch transitions.
#include <cstdint>
#include <iostream>

enum class TrafficLight { Red, Green, Yellow };
enum class Direction { North, East, South, West };
enum class OrderStatus { Pending, Paid, Shipped, Delivered, Cancelled };
enum class Token { Num, Plus, Minus, Star, Slash, LParen, RParen, End };

TrafficLight light_next(TrafficLight t) {
    switch (t) {
        case TrafficLight::Red: return TrafficLight::Green;
        case TrafficLight::Green: return TrafficLight::Yellow;
        case TrafficLight::Yellow: return TrafficLight::Red;
    }
    return TrafficLight::Red;
}

std::int64_t light_go(TrafficLight t) {
    return t == TrafficLight::Green ? 1 : 0;
}

std::int64_t light_duration(TrafficLight t) {
    switch (t) {
        case TrafficLight::Red: return 30;
        case TrafficLight::Green: return 25;
        case TrafficLight::Yellow: return 5;
    }
    return 0;
}

Direction turn_right(Direction d) {
    switch (d) {
        case Direction::North: return Direction::East;
        case Direction::East: return Direction::South;
        case Direction::South: return Direction::West;
        case Direction::West: return Direction::North;
    }
    return Direction::North;
}

Direction turn_left(Direction d) {
    switch (d) {
        case Direction::North: return Direction::West;
        case Direction::West: return Direction::South;
        case Direction::South: return Direction::East;
        case Direction::East: return Direction::North;
    }
    return Direction::North;
}

Direction opposite(Direction d) {
    switch (d) {
        case Direction::North: return Direction::South;
        case Direction::South: return Direction::North;
        case Direction::East: return Direction::West;
        case Direction::West: return Direction::East;
    }
    return Direction::North;
}

std::int64_t dir_dx(Direction d) {
    switch (d) {
        case Direction::East: return 1;
        case Direction::West: return -1;
        default: return 0;
    }
}

std::int64_t dir_dy(Direction d) {
    switch (d) {
        case Direction::North: return 1;
        case Direction::South: return -1;
        default: return 0;
    }
}

std::int64_t dir_clockwise_index(Direction d) {
    switch (d) {
        case Direction::North: return 0;
        case Direction::East: return 1;
        case Direction::South: return 2;
        case Direction::West: return 3;
    }
    return 0;
}

std::int64_t order_can_cancel(OrderStatus s) {
    return (s == OrderStatus::Pending || s == OrderStatus::Paid) ? 1 : 0;
}

OrderStatus order_next(OrderStatus s) {
    switch (s) {
        case OrderStatus::Pending: return OrderStatus::Paid;
        case OrderStatus::Paid: return OrderStatus::Shipped;
        case OrderStatus::Shipped: return OrderStatus::Delivered;
        case OrderStatus::Delivered: return OrderStatus::Delivered;
        case OrderStatus::Cancelled: return OrderStatus::Cancelled;
    }
    return s;
}

std::int64_t order_is_final(OrderStatus s) {
    return (s == OrderStatus::Delivered || s == OrderStatus::Cancelled) ? 1 : 0;
}

std::int64_t token_precedence(Token t) {
    switch (t) {
        case Token::Plus: case Token::Minus: return 1;
        case Token::Star: case Token::Slash: return 2;
        default: return 0;
    }
}

std::int64_t token_is_operator(Token t) {
    switch (t) {
        case Token::Plus: case Token::Minus: case Token::Star: case Token::Slash: return 1;
        default: return 0;
    }
}

std::int64_t token_is_paren(Token t) {
    return (t == Token::LParen || t == Token::RParen) ? 1 : 0;
}

const char *light_name(TrafficLight t) {
    switch (t) {
        case TrafficLight::Red: return "RED";
        case TrafficLight::Green: return "GREEN";
        case TrafficLight::Yellow: return "YELLOW";
    }
    return "?";
}

int main() {
    std::cout << "light_next=" << light_name(light_next(TrafficLight::Red)) << "\n";
    std::cout << "light_go=" << light_go(TrafficLight::Green) << "\n";
    std::cout << "light_duration=" << light_duration(TrafficLight::Red) << "\n";
    std::cout << "turn_right(N)=clockwise " << dir_clockwise_index(turn_right(Direction::North)) << "\n";
    std::cout << "turn_left(N)=clockwise " << dir_clockwise_index(turn_left(Direction::North)) << "\n";
    std::cout << "opposite(N)=clockwise " << dir_clockwise_index(opposite(Direction::North)) << "\n";
    std::cout << "dir_dx(East)=" << dir_dx(Direction::East) << "\n";
    std::cout << "dir_dy(South)=" << dir_dy(Direction::South) << "\n";
    std::cout << "order_can_cancel(Paid)=" << order_can_cancel(OrderStatus::Paid) << "\n";
    std::cout << "order_is_final(order_next(Shipped))=" << order_is_final(order_next(OrderStatus::Shipped)) << "\n";
    std::cout << "token_precedence(Star)=" << token_precedence(Token::Star) << "\n";
    std::cout << "token_is_operator(Plus)=" << token_is_operator(Token::Plus) << "\n";
    std::cout << "token_is_paren(LParen)=" << token_is_paren(Token::LParen) << "\n";
    return 0;
}
