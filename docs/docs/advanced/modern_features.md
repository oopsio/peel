# Modern Features in Peel

Peel has recently integrated an array of modern language features inspired by contemporary programming workflows.

## Classes with Getters, Setters, and Methods

Peel supports class definitions with encapsulated state methods, getters, and setters.
Classes provide a blueprint for creating objects, making object-oriented patterns straightforward.

```peel
class Person {
    fn init(name, age) {
        self.name = name;
        self.age = age;
    }

    get info() {
        return self.name + " (" + self.age + ")";
    }

    set info(new_age) {
        self.age = new_age;
    }

    fn greet() {
        fmt.println("Hi, I am " + self.info);
    }
}
```

## Rest, Spread, and Default Parameters

Functions in Peel can now specify **default parameter values** to omit repetitive arguments. 
Additionally, **rest parameters (`...`)** collect any excess arguments into a List, and the **spread operator (`...`)** can expand Lists into function arguments or other Lists.

```peel
fn make_team(leader = "Alice", ...members) {
    fmt.println("Leader:", leader);
    fmt.println("Members:", members);
}

let extra_members = ["Charlie", "Diana"];
make_team("Bob", "Eve", ...extra_members);
```

## Optional Chaining (`?.`) and Nullish Coalescing (`??`)

Working with nested values or potentially undefined fields is safer with **Optional Chaining** and **Nullish Coalescing**.

*   `?.` allows accessing nested fields safely. If any link is missing, it evaluates to void without throwing.
*   `??` returns the right-side operand when its left side is void, None, or undefined.

```peel
let config = { settings: { theme: "dark" } };
let theme = config?.settings?.theme ?? "light";
let font = config?.settings?.font ?? "Arial";
```

## Advanced Collections

Peel's standard library now offers native `Map`, `Set`, `WeakMap`, and `WeakSet`.

```peel
// Map and Set
let m = Map();
m.set("key", "value");

let s = Set();
s.add("unique");

// WeakMap and WeakSet (keys must be objects/lists)
let wm = WeakMap();
let objKey = { id: 1 };
wm.set(objKey, "data");
```

## Generators and Iterators

Peel will support generators to pause and resume execution using `yield` combined with channels or async tasks.

```peel
// Grammar support available
yield "value"; 
```
