use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;


#[macro_use]
extern crate lazy_static;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Rational {
    numerator: i64,
    denominator: i64,
}

#[derive(Debug, PartialEq)]
pub enum RationalParseError {
    UnexpectedEndOfLine,
    InvalidNumber,
    NumberExpected,
    InvalidCharacter(char),
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

impl fmt::Display for Rational {
    /// Displays a rational number as string
    ///
    /// If the rational number has a denominator other than 1, only the numerator is output as a character string.
    /// Otherwise the rational number is output as a fraction in the form *numerator/denominator*
    ///
    /// # Examples
    ///
    /// ```
    /// use recipers::Rational;
    ///
    /// let rational = Rational::new(42, 5);
    /// let formatted = format!("{}", rational);
    /// assert_eq!(formatted, "42/5")
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.denominator == 1 {
            write!(f, "{}", self.numerator)
        } else {
            write!(f, "{}/{}", self.numerator, self.denominator)
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct MixedFraction {
    sign: i64,
    number: u64,
    numerator: u64,
    denominator: u64
}

enum ParseState {
    Q0, // Start
    Q1(MixedFraction), // Sign
    Q2(MixedFraction), // Numerator
    Q3(MixedFraction), // Fraction Bar
    Q4(MixedFraction), // Denominator
    Q5(MixedFraction), // Unicode Symbol
    Q6(MixedFraction),
    Q7(MixedFraction),
}

lazy_static! {
    static ref FRACTION_MAP: HashMap<char, Rational> = {
        let mut map = HashMap::new();
        map.insert('\u{00bd}', rat!(1, 2));
        map.insert('\u{2153}', rat!(1, 3));
        map.insert('\u{2154}', rat!(2, 3));
        map.insert('\u{00bc}', rat!(1, 4));
        map.insert('\u{00be}', rat!(3, 4));
        map.insert('\u{2155}', rat!(1, 5));
        map.insert('\u{2156}', rat!(2, 5));
        map.insert('\u{2157}', rat!(3, 5));
        map.insert('\u{2158}', rat!(4, 5));
        map.insert('\u{2159}', rat!(1, 6));
        map.insert('\u{215a}', rat!(5, 6));
        map.insert('\u{2150}', rat!(1, 7));
        map.insert('\u{215b}', rat!(1, 8));
        map.insert('\u{215c}', rat!(3, 8));
        map.insert('\u{215d}', rat!(5, 8));
        map.insert('\u{215e}', rat!(7, 8));
        map.insert('\u{2151}', rat!(1, 9));
        map.insert('\u{2152}', rat!(1, 10));
    
        map
    };
}

fn is_fraction_symbol(c: &char) -> bool {    
    FRACTION_MAP.contains_key(c)
}



impl FromStr for Rational {
    type Err = RationalParseError;

    /// Parses a rational number from a string.
    ///
    /// If parsing succeeds, returns the parsed rational number inside
    /// Ok, otherwise when the string is ill-formed return an error of
    /// type RationalParseError inside Err.
    ///
    /// # Examples
    ///
    /// ```
    /// use recipers::Rational;
    /// use std::str::FromStr;
    /// 
    /// let number = Rational::from_str("1/2").unwrap();
    /// 
    /// assert_eq!(number, Rational::new(1, 2));
    /// ```
    ///
    /// ```
    /// use recipers::Rational;
    /// 
    /// let number: Rational = "-1/2".parse().unwrap();
    /// assert_eq!(number, Rational::new(-1, 2));
    /// 
    /// ```
    /// # DFA definition
    /// Q = {q0, q1, q2, q3, q4}
    /// Σ = {"1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "+", "-", "/"}
    ///
    /// δ: Q x Σ -> Q (Übergangsfunktionen)
    /// |Q      |"0"-"9"| "½"-"⅒" | "+" oder "-"| "/" | " " |
    /// |-------|-------|----------|-------------|-----|-----|
    /// |q0     | q2    | q5       | q1          |     |     |
    /// |q1     | q2    | q5       |             |     |     |
    /// |q2*    | q2    |          |             | q3  | q6  |
    /// |q3     | q4    |          |             |     |     |
    /// |q4*    | q4    |          |             |     |     |
    /// |q5*    |       |          |             |     |     |
    /// |q6*    | q7    | q5       |             |     |     |
    /// |q7     | q7    |          |             | q3  |     |
    ///
    /// F = {q2, q4, q5, a6}
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut state = ParseState::Q0;

        for c in s.chars() {
            state = match c {
                f if is_fraction_symbol(&c) => {
                    let val = FRACTION_MAP.get(&f).expect("character must be a fraction");
                    match state {
                        ParseState::Q0 => ParseState::Q5(
                            MixedFraction { 
                                sign: 1, 
                                number: 0, 
                                numerator: val.numerator as u64, 
                                denominator: val.denominator as u64
                             }),
                        ParseState::Q1(sign) => ParseState::Q5(
                            MixedFraction { 
                                sign:sign.sign, 
                                number: sign.number, 
                                numerator: val.numerator as u64, 
                                denominator: val.denominator as u64,
                             }),
                             ParseState::Q2(number) => ParseState::Q5(MixedFraction { 
                                sign: number.sign, 
                                number: number.number, 
                                numerator: val.numerator as u64, 
                                denominator: val.denominator as u64,
                             }),

                        _ => return Err(RationalParseError::InvalidCharacter(c))
                    }
                }
                '0'..='9' => {
                    match state {
                        ParseState::Q0 => ParseState::Q2(MixedFraction { 
                            sign: 1, 
                            number: c.to_digit(10).unwrap() as u64, 
                            numerator: 0, 
                            denominator: 1 
                        }),
                        ParseState::Q1(sign) => ParseState::Q2( MixedFraction {
                            sign: sign.sign, 
                            number: c.to_digit(10).unwrap() as u64, 
                            numerator: 0,
                            denominator:1
                        }),
                        ParseState::Q2(number)=> ParseState::Q2(
                            MixedFraction { 
                                sign: number.sign,
                                number: number.number * 10 + c.to_digit(10).unwrap() as u64, 
                                numerator: 0, // kann noch nicht gesetzt worden sein.
                                denominator: 1 // kann noch nicht gesetzt worden sein.
                            }),
                        ParseState::Q3(number) => ParseState::Q4(
                            MixedFraction { 
                                sign: number.sign, 
                                number: number.number, 
                                numerator: number.numerator, 
                                denominator: c.to_digit(10).unwrap() as u64
                            }),
                        ParseState::Q4(fraction) => ParseState::Q4(
                            MixedFraction { 
                                sign: fraction.sign, 
                                number: fraction.number, 
                                numerator: fraction.numerator, 
                                denominator: fraction.denominator * 10 + c.to_digit(10).unwrap() as u64 
                            }),
                        _ => return Err(RationalParseError::InvalidCharacter(c)),
                    }
                },
                '+' => {
                    match state {
                        ParseState::Q0 => ParseState::Q1(MixedFraction { sign: 1, number: 0, numerator:0, denominator: 1}),
                        _ => return Err(RationalParseError::InvalidCharacter(c))
                    }
                },
                '-' => {
                    match state {
                        ParseState::Q0 => ParseState::Q1(MixedFraction { sign: -1, number: 0, numerator: 0, denominator: 1 }),
                        _ => return Err(RationalParseError::InvalidCharacter(c))
                    }
                }
                '/' => {
                    match state {
                        ParseState::Q2(number) => ParseState::Q3(
                            MixedFraction { 
                                sign: number.sign, 
                                number: 0, 
                                numerator: number.number, 
                                denominator: 0, 
                            }),
                        _ => return Err(RationalParseError::InvalidCharacter(c))
                    }
                }

                x => return Err(RationalParseError::InvalidCharacter(x)),
            }
        }

        match state {
            ParseState::Q1(_) => Err(RationalParseError::NumberExpected),
            ParseState::Q2(value) => Ok(rat!(value.sign * value.number as i64, 1)),
            ParseState::Q3(_) => Err(RationalParseError::NumberExpected),
            ParseState::Q4(value) => Ok(rat!(value.sign * value.numerator as i64, value.denominator as i64)),
            ParseState::Q5(value) => Ok(rat!(value.sign * value.number as i64 * value.denominator as i64 +  value.sign *value.numerator as i64, value.denominator as i64)),
            _ => Err(RationalParseError::UnexpectedEndOfLine),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use spucky::spec;

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
    fn display_rational() {
        struct Testcase {
            subject: Rational,
            want: String,
        }

        let testcases = [
            Testcase {
                subject: rat!(1, 2),
                want: "1/2".into(),
            },
            Testcase {
                subject: rat!(-3, 4),
                want: "-3/4".to_string(),
            },
        ];

        for testcase in testcases {
            let got = testcase.subject.to_string();
            assert_eq!(got, testcase.want)
        }
    }

    spec! {
        parse_rational {
            case case1 {
                let input = "1";
                let want = rat!(1);
            }

            case case2 {
                let input = "+1";
                let want = rat!(1);
            }

            case case3 {
                let input = "-1";
                let want = rat!(-1);
            }
            case case4 {
                let input = "42";
                let want = rat!(42);
            }

            case case5 {
                let input = "+42";
                let want = rat!(42);
            }

            case case6 {
                let input = "-42";
                let want = rat!(-42);
            }

            case case7 {
                let input = "1/2";
                let want = rat!(1, 2);
            }

            case case8 {
                let input = "+1/2";
                let want = rat!(1, 2);
            }

            case case9 {
                let input = "-1/2";
                let want = rat!(-1, 2);
            }

            case case10 {
                let input = "1111/2222";
                let want = rat!(1111, 2222);
            }

            case case11 {
                let input = "+123/124";
                let want = rat!(123, 124);
            }

            case case12 {
                let input = "-125/126";
                let want = rat!(-125, 126);
            }

            case case13 {
                let input = "\u{00bd}";
                let want = rat!(1, 2);
            }

            case case14{
                let input = "+\u{2153}";
                let want = rat!(1, 3);
            }

            case case15 {
                let input = "-\u{2154}";
                let want = rat!(-2, 3);
            }

            case case16 {
                let input = "42\u{00bd}";
                let want = rat!(42 * 2 + 1, 2);
            }

            case case17 {
                let input = "+17\u{2153}";
                let want = rat!(17 * 3 + 1, 3);
            }

            case case18 {
                let input = "-6\u{2154}";
                let want = rat!(-6 * 3 + -2, 3);
            }

            let got = input.parse().unwrap();
            assert_eq!(want, got, "want {:?}, got {:?} for input '{}'", want, got, input)
        }

    }

    spec! {
        parse_error {
            case case1 {
                let input = "";
                let want = RationalParseError::UnexpectedEndOfLine;
            }

            case case2 {
                let input = "+";
                let want = RationalParseError::NumberExpected;
            }

            case case3 {
                let input = "-";
                let want = RationalParseError::NumberExpected;
            }

            case case4 {
                let input = "+-";
                let want = RationalParseError::InvalidCharacter('-');
            }

            case case5 {
                let input = "1/";
                let want = RationalParseError::NumberExpected;
            }

            case case6 {
                let input = "1/-";
                let want = RationalParseError::InvalidCharacter('-');
            }

            case case7 {
                let input = "1/+";
                let want = RationalParseError::InvalidCharacter('+');
            }

            case case8 {
                let input = "1/a";
                let want = RationalParseError::InvalidCharacter('a');
            }

            case case9 {
                let input = "1//";
                let want = RationalParseError::InvalidCharacter('/');
            }

            let got: Result<Rational, RationalParseError> = input.parse();
            match got {
                Ok(r) => panic!("expected error, got {:?}", r),
                Err(got) => assert_eq!(want, got),
            }
        }
    }

}
