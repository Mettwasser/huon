use serde::{Deserialize, Serialize};

/*
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
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct JobInfo<'a> {
    pub pay: i64,
    #[serde(borrow)]
    pub payrate: PayRate<'a>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Job<'a> {
    pub category: JobCategory<'a>,
    pub info: JobInfo<'a>,
    pub name: &'a str,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Person<'a> {
    pub name: &'a str,
    pub first_job: Job<'a>,
    pub age: i64,
    pub second_job: Job<'a>,
    pub last_name: &'a str,
}
