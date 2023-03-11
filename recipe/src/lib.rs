use crate::rational::Rational;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod rational;
pub mod repository;

#[macro_use]
extern crate lazy_static;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
struct Ingredient {
    name: String,
    quantity: Rational,
    unit: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
struct Summary {
    title: String,
    id: Uuid,
}

impl Into<Summary> for (&Uuid, &Recipe) {
    fn into(self) -> Summary {
        Summary {
            id: *self.0,
            title: self.1.title.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TableOfContents {
    total: usize,
    content: Vec<Summary>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Recipe {
    title: String,
    #[serde(default)]
    preparation: String,
    servings: u8,
    ingredients: Vec<Ingredient>,
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::{Result, Value};

    use spucky::spec;

    spec! {
        serialize_json {
            type Output = serde_json::Result<()>
            case case0 {
                let recipe = Recipe {
                    title: "Lasagne".into(),
                    preparation: "Du weist schon wie".into(),
                    servings: 4,
                    ingredients: vec![Ingredient { name: "Pasta".into(), quantity: rat!(5, 3), unit: "pc".into()}],
                };

                let want = include_str!("fixture/lasagne.json");
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
                let json = include_str!("fixture/lasagne.json");
                let want = Recipe {
                    title: "Lasagne".into(),
                    preparation: "Du weist schon wie".into(),
                    servings: 4,
                    ingredients: vec![Ingredient {name: "Pasta".into(), quantity: rat!(5, 3), unit: "pc".into()}]
                };
            }

            let got = serde_json::from_str(json).unwrap();
            assert_eq!(want, got);
        }

    }
}
