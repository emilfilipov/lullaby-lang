// Cross-language state_machines suite (Rust). Enums + match-driven transitions.
#![allow(dead_code)]

#[derive(Clone, Copy, PartialEq)]
enum TrafficLight {
    Red,
    Green,
    Yellow,
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Copy, PartialEq)]
enum OrderStatus {
    Pending,
    Paid,
    Shipped,
    Delivered,
    Cancelled,
}

#[derive(Clone, Copy, PartialEq)]
enum Token {
    Num,
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    End,
}

fn light_next(t: TrafficLight) -> TrafficLight {
    match t {
        TrafficLight::Red => TrafficLight::Green,
        TrafficLight::Green => TrafficLight::Yellow,
        TrafficLight::Yellow => TrafficLight::Red,
    }
}

fn light_go(t: TrafficLight) -> i64 {
    match t {
        TrafficLight::Green => 1,
        _ => 0,
    }
}

fn light_duration(t: TrafficLight) -> i64 {
    match t {
        TrafficLight::Red => 30,
        TrafficLight::Green => 25,
        TrafficLight::Yellow => 5,
    }
}

fn turn_right(d: Direction) -> Direction {
    match d {
        Direction::North => Direction::East,
        Direction::East => Direction::South,
        Direction::South => Direction::West,
        Direction::West => Direction::North,
    }
}

fn turn_left(d: Direction) -> Direction {
    match d {
        Direction::North => Direction::West,
        Direction::West => Direction::South,
        Direction::South => Direction::East,
        Direction::East => Direction::North,
    }
}

fn opposite(d: Direction) -> Direction {
    match d {
        Direction::North => Direction::South,
        Direction::South => Direction::North,
        Direction::East => Direction::West,
        Direction::West => Direction::East,
    }
}

fn dir_dx(d: Direction) -> i64 {
    match d {
        Direction::East => 1,
        Direction::West => -1,
        _ => 0,
    }
}

fn dir_dy(d: Direction) -> i64 {
    match d {
        Direction::North => 1,
        Direction::South => -1,
        _ => 0,
    }
}

fn dir_clockwise_index(d: Direction) -> i64 {
    match d {
        Direction::North => 0,
        Direction::East => 1,
        Direction::South => 2,
        Direction::West => 3,
    }
}

fn order_can_cancel(s: OrderStatus) -> i64 {
    match s {
        OrderStatus::Pending | OrderStatus::Paid => 1,
        _ => 0,
    }
}

fn order_next(s: OrderStatus) -> OrderStatus {
    match s {
        OrderStatus::Pending => OrderStatus::Paid,
        OrderStatus::Paid => OrderStatus::Shipped,
        OrderStatus::Shipped => OrderStatus::Delivered,
        OrderStatus::Delivered => OrderStatus::Delivered,
        OrderStatus::Cancelled => OrderStatus::Cancelled,
    }
}

fn order_is_final(s: OrderStatus) -> i64 {
    match s {
        OrderStatus::Delivered | OrderStatus::Cancelled => 1,
        _ => 0,
    }
}

fn token_precedence(t: Token) -> i64 {
    match t {
        Token::Plus | Token::Minus => 1,
        Token::Star | Token::Slash => 2,
        _ => 0,
    }
}

fn token_is_operator(t: Token) -> i64 {
    match t {
        Token::Plus | Token::Minus | Token::Star | Token::Slash => 1,
        _ => 0,
    }
}

fn token_is_paren(t: Token) -> i64 {
    match t {
        Token::LParen | Token::RParen => 1,
        _ => 0,
    }
}

fn light_name(t: TrafficLight) -> &'static str {
    match t {
        TrafficLight::Red => "RED",
        TrafficLight::Green => "GREEN",
        TrafficLight::Yellow => "YELLOW",
    }
}

fn main() {
    println!("light_next={}", light_name(light_next(TrafficLight::Red)));
    println!("light_go={}", light_go(TrafficLight::Green));
    println!("light_duration={}", light_duration(TrafficLight::Red));
    println!("turn_right(N)=clockwise {}", dir_clockwise_index(turn_right(Direction::North)));
    println!("turn_left(N)=clockwise {}", dir_clockwise_index(turn_left(Direction::North)));
    println!("opposite(N)=clockwise {}", dir_clockwise_index(opposite(Direction::North)));
    println!("dir_dx(East)={}", dir_dx(Direction::East));
    println!("dir_dy(South)={}", dir_dy(Direction::South));
    println!("order_can_cancel(Paid)={}", order_can_cancel(OrderStatus::Paid));
    println!("order_is_final(order_next(Shipped))={}", order_is_final(order_next(OrderStatus::Shipped)));
    println!("token_precedence(Star)={}", token_precedence(Token::Star));
    println!("token_is_operator(Plus)={}", token_is_operator(Token::Plus));
    println!("token_is_paren(LParen)={}", token_is_paren(Token::LParen));
}
