# Functions

Functions are prevalent in Peel code. You use the `fn` keyword to declare a new function. Functions in Peel use _snake_case_ as the conventional style for function and variable names.

## Defining a Function

Here is a basic function that prints a greeting:

```peel
fn main() {
    println!("Hello, world!");

    another_function();
}

fn another_function() {
    println!("Another function.");
}
```

Peel execution begins in the `main` function. Other functions can be defined anywhere; as long as they are accessible in the same file or imported, they can be called.

## Parameters

We can define functions to have _parameters_, which are special variables that are part of a function's signature. When you define parameters, you must declare their type.

```peel
fn print_labeled_measurement(value: int, unit_label: string) {
    fmt.println("The measurement is:", value, unit_label);
}

fn main() {
    print_labeled_measurement(5, "h");
}
```

## Return Values

Functions can return values to the code that calls them. We don't name return values, but we must declare their type after an arrow (`->`).

```peel
fn five() -> int {
    return 5;
}

fn main() {
    let x = five();
    fmt.println("The value of x is:", x);
}
```

```peel
fn plus_one(x: int) -> int {
    return x + 1;
}
```

## Statements vs. Expressions

- **Statements** are instructions that perform some action and do not return a value. For example, `let y = 6;` is a statement.
- **Expressions** evaluate to a resultant value. Adding `5 + 6` is an expression that evaluates to `11`. Calling a function is an expression. Calling a macro is an expression. A new scope block created with curly brackets `{}` is an expression.

```peel
let y = {
    let x = 3;
    x + 1 // This expression is returned to `let y`
};
```
