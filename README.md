# HUON

> Horrible Unified Object Notation

My very own data format. This has been made for learning purposes only.

The tokenizer is entirely stack allocated, the parser has 1 exception to this: hashmaps that will represent the HUON input.

# Performance

Benchmarks are included to measure deserialization performance using `criterion`.

The benchmarks are defined in `benches/parsing.rs` and measure the time taken to deserialize:

- `test.huon`: A complex HUON structure representing a `Person` object.
- `test_list.huon`: A simpler HUON structure representing `CodeInfo`.

While specific numbers can vary by system, the parsing remains efficient due to the design, with most operations being stack-allocated.

## Benchmarking (Deserialization)

The results below were captured on an Intel(R) Core(TM) i7-13700KF powered Linux (CachyOS) machine.

| Benchmark Input File                 | Model                                         | Time       |
| ------------------------------------ | --------------------------------------------- | ---------- |
| [`test.huon`](./test.huon)           | [`test_model`](./src/test_model.rs)           | ~1.9006 µs |
| [`test_list.huon`](./test_list.huon) | [`test_list_model`](./src/test_list_model.rs) | ~468.44 ns |

# serde

Serde is mostly\* supported.

_\* maybe some edge cases aren't covered_

# Example

## Deserialization

### The model

```rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewType<'a>(pub &'a str);

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct JobCategory<'a> {
    #[serde(borrow)]
    pub name: NewType<'a>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PayRate<'a> {
    pub iteration: &'a str,
    pub date: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_increase: Option<&'a str>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct JobInfo<'a> {
    pub pay: f64,
    #[serde(borrow)]
    pub payrate: PayRate<'a>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Job<'a> {
    pub category: JobCategory<'a>,
    pub info: JobInfo<'a>,
    pub name: &'a str,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Person<'a> {
    pub name: &'a str,
    pub job1: Job<'a>,
    pub age: i64,
    pub job2: Job<'a>,
    pub last_name: &'a str,
}
```

### The input

```yaml
name: "John"
job1:
  category:
    name: "IT"
  info:
    pay: -4200.5
    payrate:
      iteration: "monthly"
      date: "Last Friday of every month"
      monthly_increase: "5%"
  name: "Software Engineer"
age: 32
job2:
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

### The test

```rs
let input = include_str!("input from above").to_owned();

let person: Person =
    from_str(&input, DecoderOptions::default()).expect("Deserialization failed");

let expected_person = Person {
    name: "John",
    last_name: "Doe",
    age: 32,
    job1: Job {
        category: JobCategory {
            name: NewType("IT"),
        },
        info: JobInfo {
            pay: -4200.50,
            payrate: PayRate {
                iteration: "monthly",
                date: "Last Friday of every month",
                monthly_increase: Some("5%"),
            },
        },
        name: "Software Engineer",
    },
    .. // omitted
};

assert_eq!(person, expected_person);
```

## Serialization (smaller example)

### The model

```rs
#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct CodeInfo {
    pub test_codes: TestCodes,
    pub name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct TestCodes {
    pub codes: Vec<f64>,
    pub info: String,
}
```

### The input

```yaml
test_codes:
  codes: [111.1 333.3 555.5]
  info: "Passwords"
name: "General Access"
```

### The test

```rs
let code_info = CodeInfo {
    test_codes: TestCodes {
        codes: vec![111.1, 333.3, 555.5],
        info: "Passwords".to_string(),
    },
    name: "General Access".to_string(),
};

let s = to_string(
    &code_info,
    EncoderOptions {
        indent: 2, // indents to use!
        list_comma_style: ListCommaStyle::None, // None / Basic / Trailing
    },
)
.unwrap();

let expected = include_str!("input from above").to_owned();

assert_eq!(s, expected);
```

More info on [`ListCommaStyle`](./src/lib.rs#L16-L25)
