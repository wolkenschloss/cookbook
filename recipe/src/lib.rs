use std::ops::{Add, Div, Mul, Sub};

use uuid::Uuid;

mod format;
mod parse;
pub mod repository;

#[macro_use]
extern crate lazy_static;

#[derive(Debug, PartialEq, Clone)]
pub struct Recipe {
    title: String,
    preparation: String,
    servings: u8,
    ingredients: Vec<Ingredient>,
}

#[derive(Debug, PartialEq, Clone)]
struct Ingredient {
    name: String,
    quantity: Rational,
    unit: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
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

#[derive(Debug)]
pub struct TableOfContents {
    total: usize,
    content: Vec<Summary>,
}

/// Rational represents a rational number indicating the quantity of
/// ingredients in a recipe.
///
/// Rational numbers can be created using the [rat] macro or the new
/// constructor function. The denominator cannot be 0.
///
/// # Examples
///
/// ```
/// use recipers::rat;
/// use recipers::Rational;
///
/// let three_half = rat!(3, 2);
/// assert_eq!("1½", three_half.to_string());
/// ```
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Rational {
    numerator: i64,
    denominator: i64,
}

/// The rat macro creates a new rational number.
///
/// In the first form, the macro expects the denominator and
/// numerator to be i64-type integers. The denominator cannot be 0.
///
///
///
/// # Panics
///
/// The creation of the rational number will panic if the
/// denominator is 0.
///
/// # Example
///
/// You can use the rat macro to convert integers to a rational
/// number.
///
/// ```
/// use recipers::rat;
/// use recipers::Rational;
///
/// let a = rat!(42);
/// let b = 42.into();
/// let c = rat!(42, 1);
/// assert_eq!(a, b);
/// assert_eq!(a, c);
/// ```
///
/// Rational numbers are equal if they are equal in their reduced
/// form.
///
/// ```
/// use recipers::rat;
/// use recipers::Rational;
///
/// let a = rat!(4, 2);
/// let b = rat!(2, 1);
///
/// assert_eq!(a, b);
/// ```
#[macro_export]
macro_rules! rat {
    ($n:expr, $d:expr) => {
        Rational::new($n, $d)
    };
    ($n:expr) => {
        Rational::new($n, 1)
    };
}

impl Rational {
    /// Neutral element related to the multiplication of rational
    /// numbers.
    pub const ONE: Rational = Self::new(1, 1);

    /// Neutral element regarding the addition of rational numbers.
    pub const ZERO: Rational = Self::new(0, 1);

    /// This function creates a new rational number from the
    /// numerator and denominator.
    ///
    ///
    /// The generated fraction is always presented in reduced form.
    /// The greatest common divisor of the numerator and denominator
    /// is 1, regardless of the values given as arguments for the
    /// numerator and denominator.
    ///
    /// The counter must not be 0.
    ///
    /// # Panics
    ///
    /// The function panics, if denominator is 0.
    pub const fn new(numerator: i64, denominator: i64) -> Rational {
        // assert_ne! cannot be used by a const fn.
        if denominator == 0 {
            panic!("The denominator cannot be 0")
        }

        let gcd = gcd(numerator, denominator);
        let sign = (numerator * denominator).signum();
        Rational {
            numerator: sign * (numerator / gcd).abs(),
            denominator: (denominator / gcd).abs(),
        }
    }

    fn normalize(self) -> Self {
        let gcd = gcd(self.numerator, self.denominator);
        let sign = (self.numerator * self.denominator).signum();
        Rational {
            numerator: sign * (self.numerator / gcd).abs(),
            denominator: (self.denominator / gcd).abs(),
        }
    }
}

impl Add for Rational {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let numerator = self.numerator * other.denominator + other.numerator * self.denominator;
        let denominator = self.denominator * other.denominator;

        Rational::new(numerator, denominator).normalize()
    }
}

impl Sub for Rational {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let numerator = self.numerator * rhs.denominator - rhs.numerator * self.denominator;
        let denominator = self.denominator * rhs.denominator;

        Rational::new(numerator, denominator).normalize()
    }
}

impl Mul for Rational {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let numerator = self.numerator * rhs.numerator;
        let denominator = self.denominator * rhs.denominator;

        Rational::new(numerator, denominator).normalize()
    }
}

impl Div for Rational {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let numerator = self.numerator * rhs.denominator;
        let denominator = self.denominator * rhs.numerator;

        Rational::new(numerator, denominator).normalize()
    }
}

impl From<i64> for Rational {
    fn from(value: i64) -> Self {
        Rational::new(value, 1)
    }
}

const fn gcd(m: i64, n: i64) -> i64 {
    let mut m = m;
    let mut n = n;

    loop {
        if m == 0 {
            return n;
        } else {
            let tmp = m;
            m = n % m;
            n = tmp
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use spucky::spec;

    use std::str::FromStr;
    spec! {
        rational_add {
            case case1 {
                let (a, b, want) = (rat!(1, 2), rat!(1, 2), rat!(1, 1));
            }

            case case2 {
                let (a, b, want) = (rat!(2), rat!(3), rat!(5));
            }

            case case3 {
                let (a, b, want) = (rat!(1, 2), rat!(-1, 3), rat!(1, 6));
            }

            let got = a + b;
            assert_eq!(want, got)
        }
    }

    spec! {
        rational_sub {
            case case1 {
                let description = "1/2 - 1/2 = 0";
                let (a, b , want) = (rat!(1, 2), rat!(1, 2), rat!(0));
            }

            case case2 {
                let description = "2 - -3 = 5";
                let (a, b, want) = (rat!(2), rat!(-3), rat!(5));
            }

            case case3 {
                let description = "1/3 - 1/5 = 2/15";
                let (a, b, want) = (rat!(1, 3), rat!(1, 5), rat!(2, 15));
            }

            case case4 {
                let description = "-1/3 - 1/5 = -8/15";
                let (a, b, want) = (rat!(-1,3), rat!(1, 5), rat!(-8, 15));
            }

            case case5 {
                let description = "1/3 - -1/5 = 8/15";
                let (a, b, want) = (rat!(1, 3), rat!(-1, 5), rat!(8, 15));
            }

            case case6 {
                let description = "2/3 - 1/2 = 1/6";
                let (a, b, want) = (rat!(2, 3), rat!(1, 2), rat!(1, 6));
            }


            let got = a - b;
            assert_eq!(want, got, "want {}, got {} in {}", want, got, description);
        }
    }

    spec! {
        rational_mul {
            case case1 {
                let (a, b, want) = (rat!(1, 2), rat!(1, 2), rat!(1, 4));
            }

            case case2 {
                let (a, b, want) = (rat!(2), rat!(3), rat!(6));
            }

            case case3 {
                let (a, b, want) = (rat!(1, 3), rat!(1, 5), rat!(1, 15));
            }

            case case4 {
                let (a, b, want) = (rat!(1, 3), rat!(-1, 5), rat!(-1, 15));
            }

            case case5 {
                let (a, b, want) = (rat!(1, 2), rat!(3, 4), rat!(3, 8));
            }


            let got = a * b;
            assert_eq!(want, got);
        }
    }

    spec! {
        rational_div {
            case case1 {
                let (a, b, want) = (rat!(8), rat!(2), rat!(4));
            }

            case case2 {
                let (a, b, want) = (rat!(2, 3), rat!(4, 5), rat!(5, 6));
            }
            case case3 {
                let (a, b, want) = (rat!(-2, 3), rat!(4, 5), rat!(-5, 6));
            }

            case case4 {
                let (a, b, want) = (rat!(2, 3), rat!(-4, 5), rat!(-5, 6));
            }

            case case5 {
                let (a, b, want) = (rat!(1, 2), rat!(3, 4), rat!(2, 3));
            }

            let got = a / b;
            assert_eq!(want, got);
        }
    }

    spec! {
        rational_eq {
            case case1 {
                let (a, b) = (rat!(2, 4), rat!(1, 2));
            }

            case case2 {
                let (a, b) = (rat!(4, 4), rat!(1));
            }

            case case3 {
                let (a, b) = (rat!(5, 2), rat!(5, 2));
            }

            case case4 {
                let (a, b) = (rat!(-5, 2), rat!(5, -2));
            }

            assert!(a == b)
        }
    }

    spec! {
        rational_ne {
            case case1 {
                let (a, b) = (rat!(123, 124), rat!(1, 2));
            }

            case case2 {
                let (a, b) = (rat!(1), rat!(2));
            }

            assert!(a!= b)
        }
    }

    spec! {
        rational_from {
            case case1 {
                let (input, want) = ("0", rat!(0));
            }

            case case2 {
                let (input, want) = ("42", rat!(42));
            }

            case case3 {
                let (input, want) = ("5/13", rat!(5, 13));
            }

            case case4 {
                let (input, want) = ("-5/13", rat!(-5, 13));
            }

            let result =  if let Ok(got) = Rational::from_str(input) {
                assert_eq!(want, got);
                Ok(())
            } else {
                Err(())
            };

            result.unwrap()
        }
    }

    #[test]
    fn from_int() {
        let a = Rational::from(42);
        assert_eq!(a, rat!(42, 1))
    }

    #[test]
    fn into_rational() {
        let a: Rational = 42.into();
        assert_eq!(a, rat!(42, 1))
    }

    #[test]
    fn into_rational_implicit() {
        assert_eq!(rat!(42, 1), 42.into())
    }

    #[test]
    #[should_panic(expected = "The denominator cannot be 0")]
    fn check_denominator() {
        rat!(1, 0);
    }
}
