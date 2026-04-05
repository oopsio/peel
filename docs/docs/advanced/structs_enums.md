# Structs and Enums

Structs and Enums are custom data types that let you create complex, logical data structures by giving names and types to varying pieces of data (Structs) or differing variants of a type (Enums).

## Structs

A struct, or _structure_, is a custom data type that lets you package together and name multiple related values that make up a meaningful group.

### Defining a Struct

```peel
struct User {
    active: bool,
    username: string,
    email: string,
    sign_in_count: int,
}
```

### Instantiating a Struct

To use a struct after we've defined it, we create an instance of that struct by specifying concrete values for each of the fields.

```peel
let mut user1 = User {
    active: true,
    username: "someusername123",
    email: "someone@example.com",
    sign_in_count: 1,
};

// Accessing fields
user1.email = "anotheremail@example.com";
```

Remember, by default variables are immutable. You must use `mut` to allow modification of struct instances.

### Methods

Methods are defined within an `impl` block and take `self` as the first parameter.

```peel
struct Rectangle {
    width: int,
    height: int,
}

impl Rectangle {
    fn area(self) -> int {
        return self.width * self.height;
    }

    async fn scale(mut self, factor: int) {
        self.width = self.width * factor;
        self.height = self.height * factor;
    }
}

fn main() {
    let rect1 = Rectangle { width: 30, height: 50 };
    fmt.println("The area of the rectangle is:", rect1.area());
}
```

## Enums

Enums allow you to define a type by enumerating its possible _variants_. Where structs give you a way of grouping together related fields and data, like a `Rectangle` with its `width` and `height`, enums give you a way of saying a value is one of a possible set of values.

```peel
enum IpAddrKind {
    V4,
    V6,
}
```

We can then create instances of each of the two variants:

```peel
let four = IpAddrKind::V4;
let six = IpAddrKind::V6;
```

### Enums with Data

An enum variant can optionally store arbitrary data of any type.

```peel
enum Message {
    Quit,
    Move { x: i32, y: i32 }, // Anonymous struct variant
    Write(String),           // Tuple variant
    ChangeColor(i32, i32, i32), // Tuple variant with multiple values
}
```

Like structs, enums can also have methods defined via `impl` blocks.
