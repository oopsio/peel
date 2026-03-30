# Traits and Generics

Generics and Traits are powerful tools in Peel that allow you to write reusable and highly abstract code that remains type-safe and performant.

## Generics

Generics act as a placeholder for any type. By using generics, you can write a function or struct that can handle `i32`, `f64`, or any custom type without having to duplicate the logic.

### Generic Functions

When defining a generic function, you place the generic parameters in angle brackets (`<T>`) right after the function name.

```peel
fn largest<T: std::cmp::PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];

    for item in list {
        if item > largest {
            largest = item;
        }
    }

    largest
}
```

### Generic Structs

Structs can also be generic. Here’s a `Point` struct that can hold values of any single type for both `x` and `y`:

```peel
struct Point<T> {
    x: T,
    y: T,
}

let int_point = Point { x: 5, y: 10 };
let float_point = Point { x: 1.0, y: 4.0 };
```

## Traits

A trait defines functionality a particular type has and can share with other types. We can use traits to define shared behavior in an abstract way. You can think of traits like interfaces in other object-oriented languages, with some differences.

### Defining a Trait

```peel
pub trait Summary {
    fn summarize(&self) -> String;
}
```

### Implementing a Trait

Now that we’ve defined the `Summary` trait's signatures, we can implement it on our custom types.

```peel
pub struct NewsArticle {
    pub headline: String,
    pub location: String,
    pub author: String,
    pub content: String,
}

impl Summary for NewsArticle {
    fn summarize(&self) -> String {
        format!("{}, by {} ({})", self.headline, self.author, self.location)
    }
}

pub struct Tweet {
    pub username: String,
    pub content: String,
    pub reply: bool,
    pub retweet: bool,
}

impl Summary for Tweet {
    fn summarize(&self) -> String {
        format!("{}: {}", self.username, self.content)
    }
}
```

### Default Implementations

We can provide a default implementation for trait methods. This allows implementers to keep the default behavior if they don't wish to override it.

```peel
pub trait Summary {
    fn summarize(&self) -> String {
        String::from("(Read more...)")
    }
}
```

### Trait Bounds

You can enforce that a generic type `T` implements a specific trait. This is known as a *trait bound*.

```peel
pub fn notify<T: Summary>(item: &T) {
    println!("Breaking news! {}", item.summarize());
}
```

This ensures we can safely call `item.summarize()` inside the function.
