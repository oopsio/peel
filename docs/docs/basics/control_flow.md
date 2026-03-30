# Control Flow

The ability to run some code depending on whether a condition is `true` and to run some code repeatedly while a condition is `true` are basic building blocks in Peel programs.

## `if` Expressions

An `if` expression allows you to branch your code depending on conditions.

```peel
let number = 3;

if number < 5 {
    println!("condition was true");
} else {
    println!("condition was false");
}
```

You can use `else if` to handle multiple conditions:

```peel
let number = 6;

if number % 4 == 0 {
    println!("number is divisible by 4");
} else if number % 3 == 0 {
    println!("number is divisible by 3");
} else if number % 2 == 0 {
    println!("number is divisible by 2");
} else {
    println!("number is not divisible by 4, 3, or 2");
}
```

Since `if` is an expression, we can use it on the right side of a `let` statement to assign the outcome to a variable:

```peel
let condition = true;
let number = if condition { 5 } else { 6 };
```

## Repetition with Loops

Peel has three kinds of loops: `loop`, `while`, and `for`.

### `loop`

The `loop` keyword tells Peel to execute a block of code over and over again forever or until you explicitly tell it to stop.

```peel
let mut counter = 0;

let result = loop {
    counter += 1;

    if counter == 10 {
        break counter * 2; // Return value from loop
    }
};

println!("The result is {}", result);
```

### `while` Conditional Loops

A program often needs to evaluate a condition within a loop. While the condition is `true`, the loop runs.

```peel
let mut number = 3;

while number != 0 {
    println!("{}!", number);
    number -= 1;
}

println!("LIFTOFF!!!");
```


