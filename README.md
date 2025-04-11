# HUON
> Horrible Unified Object Notation

My very own data format. This has been made for learning purposes only.

The tokenizer is entirely stack allocated, the parser has 1 exception to this: hashmaps that will represent the HUON input.

# serde
I do not have serde compatibility added *yet*.

However, I will do this in the future.

# Showtime, baby
Given the input from [the test file](test.huon).
```yml
name: "John"
age: 23
address:
    house: "Abyss"
    city:
        name: "Underworld"
        postal: 66666
john_is_adult: true
family:
    dad: "Devil 2"
    mom: "Devil 1"
combined_family_age: 398
```

Will output:
```rs
{
    "age": Int(
        23,
    ),
    "combined_family_age": Int(
        398,
    ),
    "name": String(
        "John",
    ),
    "john_is_adult": Boolean(
        true,
    ),
    "address": Object(
        {
            "house": String(
                "Abyss",
            ),
            "city": Object(
                {
                    "name": String(
                        "Underworld",
                    ),
                    "postal": Int(
                        66666,
                    ),
                },
            ),
        },
    ),
    "family": Object(
        {
            "mom": String(
                "Devil 1",
            ),
            "dad": String(
                "Devil 2",
            ),
        },
    ),
}
```
(by running the test in [parser/mod.rs](./src/parser/mod.rs))