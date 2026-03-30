# Console Output

The `fmt` and `console` modules provide ways to interact with the terminal output.

## `fmt.println`

The `fmt.println` function prints one or more values to the console, followed by a newline.

```peel
fn main() {
    fmt.println("Hello", "World", 42);
}
```

## `console` Object

The `console` object provides more structured logging options, similar to browser-based development.

```peel
console.log("Standard log message");
console.info("Information message");
console.warn("Warning with [WARN] prefix");
console.error("Error with [ERROR] prefix");
```

## Performance Timing

You can measure how long a block of code takes to execute using `console.time` and `console.timeEnd`.

```peel
console.time("loop");
// ... your code ...
console.timeEnd("loop"); // Prints: loop: 12ms
```

## Clearing the Console

```peel
console.clear();
```
