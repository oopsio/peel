# Variables & Mutability

In Peel, safety is paramount. One way we achieve this is through strict control over mutability. Variables in Peel are **immutable by default**, which means once a value is bound to a variable name, you cannot change that value.

## Declaration

Use the `let` keyword to declare a variable.

```peel
let x = 5;
println!("The value of x is: {}", x);
```

If you attempt to assign a new value to `x`, the compiler will throw an error:

```peel title="main.pel"
let x = 5;
x = 6; // ERROR: Cannot mutate immutable variable `x`
```

## Mutability

To declare a mutable variable, you must explicitly use the `mut` keyword. This signals your intent that the value will change, which is helpful for both the compiler and future readers of the code.

```peel
let mut y = 10;
println!("Initial y: {}", y);
y = 15;
println!("Mutated y: {}", y);
```

## Shadowing

You can declare a new variable with the same name as a previous variable. This is known as _shadowing_. Shadowing is distinct from mutability because it creates a completely new variable, potentially even of a different type, while reusing the same name.

```peel
let spaces = "   ";
let spaces = spaces.len(); // Shadows the previous string with an integer

println!("There are {} spaces.", spaces);
```

### Shadowing Scope

Shadowing can be particularly useful within nested scopes.

```peel
let x = 5;

{
    let x = x * 2;
    println!("Inner x: {}", x); // Prints 10
}

println!("Outer x: {}", x); // Prints 5
```
