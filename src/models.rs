use serde::Deserialize;

#[derive(Deserialize)]
pub struct Item {
    pub id: i32,
    pub name: String,
    pub description: String,
}