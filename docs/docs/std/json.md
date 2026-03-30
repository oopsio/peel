# The `JSON` Module

Peel makes interacting with JSON easy through the `JSON` global object.

## Parsing JSON

The `JSON.parse` function takes a string and converts it into a Peel object or value.

```peel
let raw_json = "{\"name\": \"Peel Coder\", \"level\": 99}";
let user = JSON.parse(raw_json);

fmt.println("Username:", user.name);
```

## Stringifying JSON

The `JSON.stringify` function takes any Peel value and converts it into a JSON string.

```peel
let data = {
    theme: "dark",
    auto_save: true
};

let payload = JSON.stringify(data);
fmt.println(payload); // {"theme":"dark","auto_save":true}
```

## Handling Types

- Peel `List` becomes a JSON Array.
- Peel `Object` becomes a JSON Object.
- `int` and `float` become JSON Numbers.
- `bool` and `string` are mapped directly.
- `Void` becomes `null`.
