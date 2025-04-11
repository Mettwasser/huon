#[derive(Debug, serde::Deserialize)]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub adult: bool,
    pub address: Address,
}

#[derive(Debug, serde::Deserialize)]
pub struct Address {
    pub house: String,
    pub postal: i32,
}

fn main() -> huon::Result<'static, ()> {
    Ok(())
}
