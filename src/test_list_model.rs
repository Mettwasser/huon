#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct CodeInfo {
    pub test_codes: TestCodes,
    pub name: String,
}

impl Default for CodeInfo {
    fn default() -> Self {
        Self {
            test_codes: TestCodes {
                codes: vec![111.1, 333.3, 555.5],
                info: "Passwords".to_string(),
            },
            name: "General Access".to_string(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct TestCodes {
    pub codes: Vec<f64>,
    pub info: String,
}
