use crate::rational::Rational;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod rational;
pub mod repository;

#[macro_use]
extern crate lazy_static;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
struct Ingredient {
    name: String,
    quantity: Rational,
    unit: Option<String>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, Deserialize)]
pub struct Summary {
    pub title: String,
    pub id: Uuid,
}

impl Into<Summary> for (&Uuid, &Recipe) {
    fn into(self) -> Summary {
        Summary {
            id: *self.0,
            title: self.1.title.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TableOfContents {
    pub total: u64,
    pub content: Vec<Summary>,
}

impl TableOfContents {
    pub fn empty() -> TableOfContents {
        TableOfContents {
            total: 0,
            content: vec![],
        }
    }

    pub fn some() -> TableOfContents {
        TableOfContents {
            total: 1,
            content: vec![Summary {
                id: Uuid::new_v4(),
                title: "My summary".into(),
            }],
        }
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Recipe {
    pub title: String,
    #[serde(default)]
    preparation: String,
    servings: u8,
    ingredients: Vec<Ingredient>,
}

use std::str::FromStr;

impl FromStr for Recipe {
    type Err = serde_json::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use spucky::spec;
    mod fixture;

    spec! {
        serialize_json {
            type Output = serde_json::Result<()>
            case case0 {
                let recipe = Recipe {
                    title: "Lasagne".into(),
                    preparation: "Du weist schon wie".into(),
                    servings: 4,
                    ingredients: vec![Ingredient { name: "Pasta".into(), quantity: rat!(5, 3), unit: Some("pc".into())}],
                };

                let want = fixture::LASAGNE;
            }

            let got = serde_json::to_string_pretty(&recipe).unwrap();
            println!("{}", got);
            println!("{}", want);
            assert_eq!(got, want);
        }
    }

    spec! {
        deserialize_recipe
         {
            case lasagne {
                let json = fixture::LASAGNE;
                let want = Recipe {
                    title: "Lasagne".into(),
                    preparation: "Du weist schon wie".into(),
                    servings: 4,
                    ingredients: vec![Ingredient {name: "Pasta".into(), quantity: rat!(5, 3), unit: Some("pc".into())}]
                };
            }

            let got = serde_json::from_str(json).unwrap();
            assert_eq!(want, got);
        }

    }
}
