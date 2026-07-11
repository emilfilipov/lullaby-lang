/* Cross-language state_machines suite (C). Enums + switch-driven transitions. */
#include <stdio.h>
#include <stdint.h>

typedef enum { Red, Green, Yellow } TrafficLight;
typedef enum { North, East, South, West } Direction;
typedef enum { Pending, Paid, Shipped, Delivered, Cancelled } OrderStatus;
typedef enum { Num, Plus, Minus, Star, Slash, LParen, RParen, End } Token;

TrafficLight light_next(TrafficLight t) {
    switch (t) {
        case Red: return Green;
        case Green: return Yellow;
        case Yellow: return Red;
    }
    return Red;
}

int64_t light_go(TrafficLight t) {
    return t == Green ? 1 : 0;
}

int64_t light_duration(TrafficLight t) {
    switch (t) {
        case Red: return 30;
        case Green: return 25;
        case Yellow: return 5;
    }
    return 0;
}

Direction turn_right(Direction d) {
    switch (d) {
        case North: return East;
        case East: return South;
        case South: return West;
        case West: return North;
    }
    return North;
}

Direction turn_left(Direction d) {
    switch (d) {
        case North: return West;
        case West: return South;
        case South: return East;
        case East: return North;
    }
    return North;
}

Direction opposite(Direction d) {
    switch (d) {
        case North: return South;
        case South: return North;
        case East: return West;
        case West: return East;
    }
    return North;
}

int64_t dir_dx(Direction d) {
    switch (d) {
        case East: return 1;
        case West: return -1;
        default: return 0;
    }
}

int64_t dir_dy(Direction d) {
    switch (d) {
        case North: return 1;
        case South: return -1;
        default: return 0;
    }
}

int64_t dir_clockwise_index(Direction d) {
    switch (d) {
        case North: return 0;
        case East: return 1;
        case South: return 2;
        case West: return 3;
    }
    return 0;
}

int64_t order_can_cancel(OrderStatus s) {
    return (s == Pending || s == Paid) ? 1 : 0;
}

OrderStatus order_next(OrderStatus s) {
    switch (s) {
        case Pending: return Paid;
        case Paid: return Shipped;
        case Shipped: return Delivered;
        case Delivered: return Delivered;
        case Cancelled: return Cancelled;
    }
    return s;
}

int64_t order_is_final(OrderStatus s) {
    return (s == Delivered || s == Cancelled) ? 1 : 0;
}

int64_t token_precedence(Token t) {
    switch (t) {
        case Plus: case Minus: return 1;
        case Star: case Slash: return 2;
        default: return 0;
    }
}

int64_t token_is_operator(Token t) {
    switch (t) {
        case Plus: case Minus: case Star: case Slash: return 1;
        default: return 0;
    }
}

int64_t token_is_paren(Token t) {
    return (t == LParen || t == RParen) ? 1 : 0;
}

static const char *light_name(TrafficLight t) {
    switch (t) {
        case Red: return "RED";
        case Green: return "GREEN";
        case Yellow: return "YELLOW";
    }
    return "?";
}

int main(void) {
    printf("light_next=%s\n", light_name(light_next(Red)));
    printf("light_go=%lld\n", (long long)light_go(Green));
    printf("light_duration=%lld\n", (long long)light_duration(Red));
    printf("turn_right(N)=clockwise %lld\n", (long long)dir_clockwise_index(turn_right(North)));
    printf("turn_left(N)=clockwise %lld\n", (long long)dir_clockwise_index(turn_left(North)));
    printf("opposite(N)=clockwise %lld\n", (long long)dir_clockwise_index(opposite(North)));
    printf("dir_dx(East)=%lld\n", (long long)dir_dx(East));
    printf("dir_dy(South)=%lld\n", (long long)dir_dy(South));
    printf("order_can_cancel(Paid)=%lld\n", (long long)order_can_cancel(Paid));
    printf("order_is_final(order_next(Shipped))=%lld\n", (long long)order_is_final(order_next(Shipped)));
    printf("token_precedence(Star)=%lld\n", (long long)token_precedence(Star));
    printf("token_is_operator(Plus)=%lld\n", (long long)token_is_operator(Plus));
    printf("token_is_paren(LParen)=%lld\n", (long long)token_is_paren(LParen));
    return 0;
}
