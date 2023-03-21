use serde::de;

#[allow(dead_code)]
static DATA: &'static str = include_str!("fixture/recipes.json");

#[allow(dead_code)]
pub static LASAGNE: &'static str = include_str!("fixture/lasagne.json");

#[allow(dead_code)]
pub static CHILI: &'static str = include_str!("fixture/chili.json");

#[allow(dead_code)]
pub fn all_recipes<'a, T>() -> serde_json::error::Result<Vec<T>>
where
    T: de::Deserialize<'a>,
{
    serde_json::from_str(&DATA)
}
