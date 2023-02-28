use std::ops::{Add, Div, Mul, Sub};

mod format;
mod parse;

#[macro_use]
extern crate lazy_static;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Rational {
    numerator: i64,
    denominator: i64,
}

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
    pub const ONE: Rational = Self::new(1, 1);
    pub const ZERO: Rational = Self::new(0, 1);

    pub const fn new(numerator: i64, denominator: i64) -> Rational {
        if denominator == 0 {
            panic!()
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
}
