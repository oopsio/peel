# The `Math` Module

The `Math` global object provides standard mathematical functions and constants.

## Constants

```peel
fmt.println("Pi:", Math.PI); // 3.141592...
fmt.println("E:", Math.E);   // 2.718281...
```

## Functions

Math functions accept both `int` and `float` and return a `float`.

```peel
let val = 9.0;
fmt.println("Sqrt:", Math.sqrt(val)); // 3.0
fmt.println("Pow:", Math.pow(2, 3));   // 8.0
fmt.println("Abs:", Math.abs(-42));    // 42.0

fmt.println("Sin:", Math.sin(1.0));
fmt.println("Cos:", Math.cos(1.0));
fmt.println("Tan:", Math.tan(1.0));

fmt.println("Floor:", Math.floor(3.7)); // 3.0
fmt.println("Ceil:", Math.ceil(3.2));   // 4.0
fmt.println("Round:", Math.round(3.5)); // 4.0
```

## Random Numbers

Returns a float between 0.0 and 1.0.

```peel
let r = Math.random();
```
