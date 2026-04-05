# Data Types

Peel is a statically typed language, which means that it must know the types of all variables at compile time. The compiler can usually infer what type we want to use based on the value and how we use it.

## Scalar Types

A scalar type represents a single value. Peel has four primary scalar types: integers, floating-point numbers, booleans, and strings.

### Integer Type (`int`)

Peel uses a single 64-bit signed integer type: `int`.

```peel
let a: int = 42;
let b = 100;
```

### Floating-Point Type (`float`)

Peel uses a 64-bit floating-point type: `float`.

```peel
let x = 2.0;
let y: float = 3.14;
```

### The Boolean Type (`bool`)

A boolean type has two possible values: `true` and `false`.

```peel
let t = true;
let f: bool = false;
```

### The String Type (`string`)

Peel has a native `string` type for UTF-8 text.

```peel
let msg: string = "Hello Peel!";
```

## Compound Types

Compound types can group multiple values into one type. Peel has two primitive compound types: tuples and arrays.

### The Tuple Type

A tuple is a general way of grouping together a number of values with a variety of types into one compound type.

```peel
let tup: (i32, f64, u8) = (500, 6.4, 1);

// Destructuring a tuple
let (x, y, z) = tup;
println!("The value of y is: {}", y);

// Accessing by index
let five_hundred = tup.0;
```

### The Array Type

Unlike a tuple, every element of an array must have the same type. Arrays in Peel have a fixed length.

```peel
let a = [1, 2, 3, 4, 5];

// Array with type notation: [type; length]
let b: [i32; 5] = [1, 2, 3, 4, 5];

// Initialize array with same value
let a = [3; 5]; // [3, 3, 3, 3, 3]

// Accessing elements
let first = a[0];
```

If you need a collection that can grow and shrink automatically, use a `Vector` from the standard library.
