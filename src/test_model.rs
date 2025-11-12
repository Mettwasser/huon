use serde::{Deserialize, Serialize};

/*
name: "John"
job1:
    category:
        name: "IT"
    info:
        pay: 4200
        payrate:
            iteration: "monthly"
            date: "Last Friday of every month"
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
*/

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
