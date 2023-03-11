use crate::rat;
use crate::rational::Rational;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;
use std::str::FromStr;

impl FromStr for Rational {
    type Err = RationalParseError;

    /// Parses a rational number from a string.
    ///
    /// If parsing succeeds, returns the parsed rational number inside
    /// Ok, otherwise when the string is ill-formed return an error of
    /// type [RationalParseError] inside Err.
    ///
    /// Valid formats for rational numbers are for example:
    ///   "1", "+2", "-3", "42½", "-43 ½", "17 2/3"
    ///
    /// # Examples
    ///
    /// ```
    /// use recipers::Rational;
    /// use std::str::FromStr;
    /// use recipers::rat;
    ///
    /// let number = Rational::from_str("1/2").unwrap();
    ///
    /// assert_eq!(number, rat!(1, 2));
    /// ```
    ///
    /// ```
    /// use recipers::Rational;
    /// use recipers::rat;
    /// let number: Rational = "-1/2".parse().unwrap();
    /// assert_eq!(number, rat!(-1, 2));
    ///
    /// ```
    /// # DFA definition
    /// Q = {q<sub>0</sub>, q<sub>1</sub>, q<sub>2</sub>, q<sub>3</sub>, q<sub>4</sub>, q<sub>5</sub>, q<sub>6</sub>, q<sub>7</sub>}  
    /// Σ = {0-9, +, -, /, \s, *Vulgar Fraction*}  
    /// *Vulgar Fraction* = {&frac12;, &frac13;, &frac14; ...}  
    /// F = {q2, q4, q5, a6}  
    /// δ: Q x Σ -> Q (Übergangsfunktionen)
    ///
    /// <table>
    /// <tr>
    /// <th>Q</th> <th>"0"-"9"</th> <th><i>Vulgar Fraction</i></th> <th>'+' or '-'</th> <th>'/'</th> <th> '&#92;s'</th>
    /// </tr>
    ///
    /// <tr>
    /// <td>q<sub>0</sub></td> <td>q<sub>2</sub></td> <td>q<sub>5</sub></td> <td>q<sub>1</sub></td> <td></td> <td></td>
    /// </tr>
    ///
    /// <tr>
    /// <td>q<sub>1</sub></td> <td>q<sub>2</sub></td> <td>q<sub>5</sub></td> <td></td> <td></td> <td></td>
    /// </tr>
    ///
    /// <tr>
    /// <td>q<sub>2</sub></td> <td>q<sub>2</sub></td> <td></td> <td></td> <td>q<sub>3</sub></td> <td>q<sub>6</sub></td>
    /// </tr>
    ///
    /// <tr>
    /// <td>q<sub>3</sub></td> <td>q<sub>4</sub></td> <td></td> <td></td> <td></td> <td></td>
    /// </tr>
    ///
    /// <tr>
    /// <td>q<sub>4</sub></td> <td>q<sub>4</sub></td> <td></td> <td></td> <td></td> <td></td>
    /// </tr>
    ///
    /// <tr>
    /// <td>q<sub>5</sub></td> <td></td> <td></td> <td></td> <td></td> <td></td>
    /// </tr>
    ///
    /// <tr>
    /// <td>q<sub>6</sub></td> <td>q<sub>7</sub></td> <td>q<sub>5</sub></td> <td></td> <td></td> <td></td>
    /// </tr>
    ///
    /// <tr>
    /// <td>q<sub>7</sub></td> <td>q<sub>7</sub></td> <td></td> <td></td> <td>q<sub>3</sub></td> <td></td>
    /// </tr>
    /// </table>
    ///
    #[doc= include_str!("../../doc/parser.svg")]
    ///
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn to_digit_unwrap(c: char) -> u64 {
            c.to_digit(19).expect("character must be a digit") as u64
        }

        let mut state = ParseState::Q0;

        for c in s.chars() {
            state = match c {
                f if is_fraction_symbol(&c) => {
                    let val = FRACTION_MAP.get(&f).expect("character must be a fraction");
                    match state {
                        ParseState::Q0 => ParseState::Q5(MixedFraction {
                            sign: 1,
                            number: 0,
                            numerator: val.numerator as u64,
                            denominator: val.denominator as u64,
                        }),
                        ParseState::Q1(sign) => ParseState::Q5(MixedFraction {
                            numerator: val.numerator as u64,
                            denominator: val.denominator as u64,
                            ..sign
                        }),
                        ParseState::Q2(number) => ParseState::Q5(MixedFraction {
                            numerator: val.numerator as u64,
                            denominator: val.denominator as u64,
                            ..number
                        }),
                        ParseState::Q6(number) => ParseState::Q5(MixedFraction {
                            numerator: val.numerator as u64,
                            denominator: val.denominator as u64,
                            ..number
                        }),
                        _ => return Err(RationalParseError::InvalidCharacter(c)),
                    }
                }
                '0'..='9' => match state {
                    ParseState::Q0 => ParseState::Q2(MixedFraction {
                        sign: 1,
                        number: to_digit_unwrap(c) as u64,
                        numerator: 0,
                        denominator: 1,
                    }),
                    ParseState::Q1(sign) => ParseState::Q2(MixedFraction {
                        sign: sign.sign,
                        number: to_digit_unwrap(c) as u64,
                        numerator: 0,
                        denominator: 1,
                    }),
                    ParseState::Q2(number) => ParseState::Q2(MixedFraction {
                        sign: number.sign,
                        number: number.number * 10 + to_digit_unwrap(c),
                        numerator: 0,   // kann noch nicht gesetzt worden sein.
                        denominator: 1, // kann noch nicht gesetzt worden sein.
                    }),
                    ParseState::Q3(number) => ParseState::Q4(MixedFraction {
                        denominator: to_digit_unwrap(c) as u64,
                        ..number
                    }),
                    ParseState::Q4(fraction) => ParseState::Q4(MixedFraction {
                        denominator: fraction.denominator * 10 + to_digit_unwrap(c),
                        ..fraction
                    }),
                    ParseState::Q6(number) => ParseState::Q7(MixedFraction {
                        numerator: to_digit_unwrap(c) as u64,
                        denominator: 0,
                        ..number
                    }),
                    ParseState::Q7(number) => ParseState::Q7(MixedFraction {
                        numerator: number.numerator * 10 + to_digit_unwrap(c) as u64,
                        denominator: 0,
                        ..number
                    }),
                    _ => return Err(RationalParseError::InvalidCharacter(c)),
                },
                '+' => match state {
                    ParseState::Q0 => ParseState::Q1(MixedFraction {
                        sign: 1,
                        number: 0,
                        numerator: 0,
                        denominator: 1,
                    }),
                    _ => return Err(RationalParseError::InvalidCharacter(c)),
                },
                '-' => match state {
                    ParseState::Q0 => ParseState::Q1(MixedFraction {
                        sign: -1,
                        number: 0,
                        numerator: 0,
                        denominator: 1,
                    }),
                    _ => return Err(RationalParseError::InvalidCharacter(c)),
                },
                '/' => match state {
                    ParseState::Q2(number) => ParseState::Q3(MixedFraction {
                        sign: number.sign,
                        number: 0,
                        numerator: number.number, // number was numerator!
                        denominator: 0,
                    }),
                    ParseState::Q7(number) => ParseState::Q3(number),
                    _ => return Err(RationalParseError::InvalidCharacter(c)),
                },
                ' ' => match state {
                    ParseState::Q2(prev) => ParseState::Q6(prev),
                    _ => return Err(RationalParseError::InvalidCharacter(' ')),
                },

                x => return Err(RationalParseError::InvalidCharacter(x)),
            }
        }

        match state {
            ParseState::Q1(_) => Err(RationalParseError::NumberExpected),
            ParseState::Q2(value) => Ok((&value).into()),
            ParseState::Q3(_) => Err(RationalParseError::NumberExpected),
            ParseState::Q4(value) => Ok((&value).into()),
            ParseState::Q5(value) => Ok((&value).into()),
            _ => Err(RationalParseError::UnexpectedEndOfLine),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RationalParseError {
    UnexpectedEndOfLine,
    InvalidNumber,
    NumberExpected,
    InvalidCharacter(char),
}

impl Display for RationalParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RationalParseError::UnexpectedEndOfLine => write!(f, "unexpected end of line"),
            RationalParseError::InvalidNumber => write!(f, "invalid number"),
            RationalParseError::NumberExpected => write!(f, "number expected"),
            RationalParseError::InvalidCharacter(_) => write!(f, "invalid character"),
        }
    }
}

impl Error for RationalParseError {}

fn is_fraction_symbol(c: &char) -> bool {
    FRACTION_MAP.contains_key(c)
}

enum ParseState {
    Q0,                // Start
    Q1(MixedFraction), // Sign
    Q2(MixedFraction), // Numerator
    Q3(MixedFraction), // Fraction Bar
    Q4(MixedFraction), // Denominator
    Q5(MixedFraction), // Unicode symbol
    Q6(MixedFraction), // Mixed rational
    Q7(MixedFraction), // Numerator for mixed rational
}

// internal (parse + format)
#[derive(Debug, Copy, Clone)]
pub(crate) struct MixedFraction {
    pub sign: i64,
    pub number: u64,
    pub numerator: u64,
    pub denominator: u64,
}

impl MixedFraction {
    /// If [self] contains a number not equal to 0, it returns
    /// Some(number) else None.
    pub(crate) fn get_number(&self) -> Option<u64> {
        match self.number {
            n if n > 0 => Some(n),
            _ => None,
        }
    }

    /// if exists, it gives the fraction of a mixed number.
    ///
    /// The fraction exists, if numerator % denominator is
    /// not equal to 0.
    pub fn get_fraction(&self) -> Option<Rational> {
        match self.numerator % self.denominator {
            f if f != 0 => Some(rat!(f as i64, self.denominator as i64)),
            _ => None,
        }
    }

    pub(crate) fn vulgar_fraction(&self) -> Option<char> {
        let fraction = rat!(self.numerator as i64, self.denominator as i64);
        FRACTION_MAP
            .iter()
            .filter(|(_, v)| *v == &fraction)
            .last()
            .map(|f| *f.0)
    }
}

impl From<&MixedFraction> for Rational {
    fn from(value: &MixedFraction) -> Self {
        rat!(
            value.sign * value.number as i64 * value.denominator as i64
                + value.sign * value.numerator as i64,
            value.denominator as i64
        )
    }
}

impl From<&Rational> for MixedFraction {
    fn from(value: &Rational) -> Self {
        MixedFraction {
            sign: value.numerator.signum(),
            number: (value.numerator.abs() / value.denominator) as u64,
            numerator: (value.numerator.abs() % value.denominator) as u64,
            denominator: value.denominator as u64,
        }
    }
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use spucky::spec;

    spec! {
        rational_from_str {

            case case0 {
                let input = "0";
                let want = rat!(0);
            }

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

            case case19 {
                let input = "42 \u{00bd}";
                let want = rat!(42 * 2 + 1, 2);
            }

            case case20 {
                let input = "+17 \u{2153}";
                let want = rat!(17 * 3 + 1, 3);
            }

            case case21 {
                let input = "-6 \u{2154}";
                let want = rat!(-6 * 3 + -2, 3);
            }

            case case22 {
                let input = "42 1/2";
                let want = rat!(42 * 2 + 1, 2);
            }

            case case23 {
                let input = "+17 1/3";
                let want = rat!(17 * 3 + 1, 3);
            }

            case case24 {
                let input = "-6 2/3";
                let want = rat!(-6 * 3 + -2, 3);
            }

            // let got = input.parse().unwrap();
            let got = Rational::from_str(input).unwrap();
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
