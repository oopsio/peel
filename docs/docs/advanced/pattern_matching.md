# Pattern Matching

Pattern matching is an incredibly powerful control flow operator in Peel that allows you to compare a value against a series of patterns and then execute code based on which pattern matches.

Think of it like a highly evolved `switch` statement that can destructure data types, enforce exhaustiveness, and bind variables beautifully.

## The `match` Operator

The `match` expression requires you to list all possible patterns (exhaustiveness).

```peel
enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter,
}

fn value_in_cents(coin: Coin) -> int {
    match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter => 25,
    }
}
```

## Patterns that Bind to Values

Another useful feature of match arms is that they can bind to the parts of the values that match the pattern. This is how we can extract values out of enum variants.

```peel
enum UsState {
    Alabama,
    Alaska,
}

enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter(UsState),
}

fn value_in_cents(coin: Coin) -> int {
    match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter(state) => {
            fmt.println("State quarter from:", state);
            25
        }
    }
}
```

## The Fallback Pattern (`*`)

When you don't care about the remaining options, you can use the wildcard `*`.

```peel
let dice_roll = 9;
match dice_roll {
    3 => add_fancy_hat(),
    7 => remove_fancy_hat(),
    * => reroll(), // Matches anything else
}
```
