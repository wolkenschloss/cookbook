use std::fmt;
use std::fmt::Formatter;
use std::ops::{Add, Div, Mul, Sub};
use std::str::FromStr;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Rational {
    numerator: i64,
    denominator: i64,
}

pub enum Token {
    Number(i64),
    Sign(char),
    Slash,
}

#[derive(Debug, PartialEq)]
pub enum RationalParseError {
    UnexpectedEndOfLine,
    InvalidNumber,
    NumberExpected,
}

#[derive(Debug)]
pub enum LexError {
    Overflow,
    InvalidChar(char),
}

impl Rational {
    const ONE: Rational = Self::new(1, 1);
    const ZERO: Rational = Self::new(0, 1);

    pub const fn new(numerator: i64, denominator: i64) -> Rational {
        if denominator == 0 {
            panic!()
        }

        let gcd = gcd(numerator, denominator);
        Rational { numerator: numerator / gcd, denominator: denominator / gcd }
    }

    fn normalize(self) -> Self {
        let gcd = gcd(self.numerator, self.denominator);
        Rational {
            numerator: self.numerator / gcd,
            denominator: self.denominator / gcd,
        }
    }

    pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
        let mut result: Vec<Token> = Vec::new();
        let mut current_number = String::new();


        for c in input.chars() {
            if !current_number.is_empty() && !(c >= '0' && c <= '9') {
                let num: i64 = match current_number.parse() {
                    Err(_) => return Err(LexError::Overflow),
                    Ok(v) => v,
                };

                result.push(Token::Number(num));
                current_number.clear()
            }

            let token = match c {
                '/' => Token::Slash,
                c @ '+' | c @ '-' => Token::Sign(c),
                n @ '0'..='9' => {
                    current_number.push(n);
                    continue;
                }
                c => return Err(LexError::InvalidChar(c)),
            };

            result.push(token)
        }

        if !current_number.is_empty() {
            let num: i64 = match current_number.parse() {
                Err(_) => return Err(LexError::Overflow),
                Ok(v) => v,
            };

            result.push(Token::Number(num))
        }

        Ok(result)
    }


    /// This function parses a rational number
    ///
    /// # Arguments
    ///
    /// * `tokens` A token slice that holds tokens to be parsed
    ///
    /// # Examples
    ///
    /// ```
    /// use recipers::Rational;
    ///
    /// let tokens = Rational::tokenize("-1/2").unwrap();
    /// let r = Rational::parse(&tokens).unwrap();
    ///
    /// assert_eq!(r, Rational::new(-1, 2))
    /// ```
    ///
    /// # BNF
    ///
    /// ```text
    /// rational
    ///  : sign number fraction ;; parse_with_sign
    ///  | number fraction      ;; parse_without_sign
    ///  ;
    ///
    /// fraction
    ///   : slash number        ;; parse_fraction
    ///   | <empty>             ;; parse_empty
    ///   ;
    ///
    /// number
    ///   : digit+              ;; parse_number
    /// ```
    pub fn parse(tokens: &[Token]) -> Result<Rational, RationalParseError> {
        if tokens.is_empty() {
            return Err(RationalParseError::UnexpectedEndOfLine);
        }

        // ggf. Sign als erstes Token:
        // fn parse_with_sign(tokens: &[Token]) -> Result<Rational, RationalParseError> {
        //   if let Sign(c) = tokens[0] {
        //      ...
        //   }
        // }
        fn parse_with_sign(c: char, tokens: &[Token]) -> Result<Rational, RationalParseError> {
            if c == '-' {
                let sign = Rational::from(-1);
                let rational = parse_without_sign(tokens)?;
                Ok(sign * rational)
            } else {
                parse_without_sign(tokens)
            }
        }

        fn parse_number(tokens: &mut &[Token]) -> Result<i64, RationalParseError> {
            if tokens.is_empty() {
                return Err(RationalParseError::NumberExpected);
            }

            return match tokens[0] {
                Token::Number(n) => {
                    *tokens = &tokens[1..];
                    Ok(n)
                }
                _ => Err(RationalParseError::NumberExpected)
            };
        }

        fn parse_fraction(tokens: &mut &[Token]) -> Result<i64, RationalParseError> {
            let mut tokens = tokens;

            if tokens.is_empty() {
                return Ok(1);
            }

            if let Token::Slash = tokens[0] {
                *tokens = &tokens[1..];
                parse_number(&mut tokens)
            } else {
                Ok(1)
            }
        }

        fn parse_without_sign(tokens: &[Token]) -> Result<Rational, RationalParseError> {
            let mut tokens = tokens;
            let numerator = {
                parse_number(&mut tokens)?
            };

            let denominator = parse_fraction(&mut tokens)?;
            Ok(Rational::new(numerator, denominator))
        }

        match tokens[0] {
            Token::Sign(c) => parse_with_sign(c, &tokens[1..]),
            _ => parse_without_sign(tokens)
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
    /// If the rational number has a denominator other than 0, only the numerator is output as a character string.
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


impl FromStr for Rational {
    type Err = RationalParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(tokens) = Rational::tokenize(s) {
            return Rational::parse(&tokens);
        } else {
            Err(RationalParseError::InvalidNumber)
        }
    }
}


#[cfg(test)]
mod test {
    use test_case::test_case;

    use spucky::spec;

    use super::*;

    spec! {
        my_suite {
            case a {
                let a = 1;
                let want = 1;
            }

            let got = a * a;
            assert_eq!(want, got);
        }
    }

    mod inner_test {
        #[test]
        fn some_inner_test() {}
    }

    #[test]
    fn add_rational() {
        let a = Rational::ONE;
        let b = Rational::new(1, 2);

        assert_eq!(a + b, Rational::new(3, 2));
    }

    #[test]
    fn sub_rational() {
        let a = Rational::ONE;
        let b = Rational::new(1, 2);

        assert_eq!(a - b, Rational::new(1, 2));
    }

    #[test]
    fn mul_rational() {
        let a = Rational::ONE;
        let b = Rational::new(1, 2);

        assert_eq!(a * b, Rational::new(1, 2));
    }

    #[test]
    fn div_rational() {
        let a = Rational::ONE;
        let b = Rational::new(1, 2);

        assert_eq!(a / b, Rational::new(2, 1));
    }

    #[test]
    fn from_int() {
        let a = Rational::from(42);
        assert_eq!(a, Rational::new(42, 1))
    }

    #[test]
    fn into_rational() {
        let a: Rational = 42.into();
        assert_eq!(a, Rational::new(42, 1))
    }

    #[test]
    fn into_rational_implicit() {
        assert_eq!(Rational::new(42, 1), 42.into())
    }

    #[test]
    fn display_rational() {
        struct Testcase {
            subject: Rational,
            want: String,
        }

        let testcases = [
            Testcase { subject: Rational::new(1, 2), want: "1/2".into() },
            Testcase { subject: Rational::new(-3, 4), want: "-3/4".to_string() }
        ];

        for testcase in testcases {
            let got = testcase.subject.to_string();
            assert_eq!(got, testcase.want)
        }
    }


    #[test]
    fn parse_rational2() {
        struct Subject<'a> {
            input: &'a str,
        }

        struct Testcase<'a> {
            name: &'static str,
            subject: Subject<'a>,
            want: Rational,
        }

        // Given
        let testcases = vec![Testcase {
            name: "one",
            subject: Subject { input: "1" },
            want: Rational::new(1, 1),
        }, Testcase {
            name: "two",
            subject: Subject { input: "2" },
            want: Rational::new(2, 1),
        }];

        // let want = vec![Rational::new(1, 1), Rational::new(2, 1)];

        fn when(testcase: &Subject) -> Result<Rational, RationalParseError> {
            testcase.input.parse()
        }

        fn then((got, want): (Result<Rational, RationalParseError>, &Rational)) -> bool {
            match got {
                Ok(r) => r == *want,
                Err(_) => false
            }
        }

        let retval: Vec<bool> = testcases.iter().map(|t| &t.subject)
            // .map(|x| -> Result<Rational, RationalParseError>{
            //     x.input.parse()
            // })
            .map(when)
            .zip(testcases.iter().map(|t| &t.want))
            .map(then)
            .collect();

        assert!(retval.into_iter().all(|r| r == true))
    }


    #[test_case("1" => Rational::new(1, 1); "1")]
    #[test_case("+1" => Rational::new(1, 1); "plus 1")]
    #[test_case("-1" => Rational::new(- 1, 1); "minus 1")]
    #[test_case("+42" => Rational::new(42, 1); "plus 42")]
    #[test_case("-42" => Rational::new(- 42, 1); "minus 42")]
    #[test_case("1/2" => Rational::new(1, 2); "a half")]
    #[test_case("+1/2" => Rational::new(1, 2); "plus a half")]
    #[test_case("-3/4" => Rational::new(- 3, 4); "minus three quarters")]
    #[test_case("1111/2222" => Rational::new(1111, 2222); "big")]
    fn parse_rational(subject: &str) -> Rational {
        subject.parse().expect("Parsing must be successful")
    }

    #[test_case("" => RationalParseError::UnexpectedEndOfLine; "when input is empty")]
    #[test_case("+" => RationalParseError::NumberExpected; "when input contains a plus sign only")]
    #[test_case("-" => RationalParseError::NumberExpected; "when input contains a minus sign only")]
    #[test_case("+-" => RationalParseError::NumberExpected; "when input contains a plus and minus sign only")]
    #[test_case("1/" => RationalParseError::NumberExpected; "when input ends with a slash")]
    #[test_case("1/-" => RationalParseError::NumberExpected; "when input contains no number after slash")]
    #[test_case("1/-" => RationalParseError::NumberExpected; "when input contains slash only")]
    fn rational_parse_error2(input: &str) -> RationalParseError {
        let tokens = Rational::tokenize(input).unwrap();
        let result = Rational::parse(&tokens);
        return result.expect_err("expecting error");
    }
}