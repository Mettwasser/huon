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
