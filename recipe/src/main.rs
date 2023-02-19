use recipers::Rational;

fn main() {
    println!("Hello, world!");
}

struct Recipe {
    title: String,
    preparation: String,
    servings: u8,
    ingredients: Vec<Ingredient>,
}

struct Ingredient {
    name: String,
    quantity: Rational,
    unit: String,
}



