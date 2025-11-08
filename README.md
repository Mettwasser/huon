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
first_job:
    category:
        name: "IT"
    info:
        pay: 4200
        payrate:
            iteration: "monthly"
            date: "Last Friday of every month"
    name: "Software Engineer"
age: 32
second_job:
    category:
        name: "Security"
    info:
        pay: 3700
        payrate:
            iteration: "weekly"
            date: "Every Friday"
    name: "Bodyguard"
last_name: "Doe"
```

Will output:
```rs
{
    "name": String(
        "John",
    ),
    "age": Int(
        32,
    ),
    "second_job": Object(
        {
            "name": String(
                "Bodyguard",
            ),
            "category": Object(
                {
                    "name": String(
                        "Security",
                    ),
                },
            ),
            "info": Object(
                {
                    "pay": Int(
                        3700,
                    ),
                    "payrate": Object(
                        {
                            "iteration": String(
                                "weekly",
                            ),
                            "date": String(
                                "Every Friday",
                            ),
                        },
                    ),
                },
            ),
        },
    ),
    "first_job": Object(
        {
            "category": Object(
                {
                    "name": String(
                        "IT",
                    ),
                },
            ),
            "info": Object(
                {
                    "payrate": Object(
                        {
                            "date": String(
                                "Last Friday of every month",
                            ),
                            "iteration": String(
                                "monthly",
                            ),
                        },
                    ),
                    "pay": Int(
                        4200,
                    ),
                },
            ),
            "name": String(
                "Software Engineer",
            ),
        },
    ),
    "last_name": String(
        "Doe",
    ),
}
```