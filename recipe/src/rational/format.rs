use crate::rational::parse::MixedFraction;
use crate::rational::Rational;
use std::fmt;

impl fmt::Display for Rational {
    /// Displays a rational number as string
    ///
    /// If the rational number has a denominator other than 1, only
    /// the numerator is output as a character string. Otherwise the
    /// rational number is output as a fraction in the form
    /// *numerator/denominator*
    ///
    /// # Examples
    ///
    /// ```
    /// use recipers::rational::Rational;
    ///
    /// let rational = Rational::new(42, 5);
    /// let formatted = format!("{}", rational);
    /// assert_eq!(formatted, "8⅖")
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mixed = MixedFraction::from(self);

        if mixed.sign < 0 {
            write!(f, "-")?
        }

        if let Some(number) = mixed.get_number() {
            write!(f, "{}", number)?;

            // print space if there is a fraction which is not a
            // vulgar fraction.
            if let (Some(_), None) = (mixed.get_fraction(), mixed.vulgar_fraction()) {
                write!(f, " ")?;
            }
        }

        if let Some(v) = mixed.vulgar_fraction() {
            write!(f, "{}", v)
        } else if let Some(r) = mixed.get_fraction() {
            write!(f, "{}/{}", r.numerator, r.denominator)
        } else {
            Ok(())
        }

        // if mixed.has_vulgar_fraction() {
        //     write!(f, "{}", mixed.vulgar_fraction())?
        // } else {
        //     write!(f, "{}/{}", mixed.numerator, mixed.denominator)?
        // }
    }
}

// impl fmt::Debug for Rational {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rat;
    use spucky::spec;

    spec! {
        display_rational {
            case case1 {
                let number = rat!(1, 2);
                let want = "½";
            }

            case case2 {
                let number = rat!(7, 2);
                let want = "3½";
            }

            case case3 {
                let number = rat!(-7, 2);
                let want = "-3½";
            }

            case case4 {
                let number = rat!(112, 11);
                let want = "10 2/11";
            }

            case case5 {
                let number = rat!(-112, 11);
                let want = "-10 2/11";
            }

            let got = number.to_string();
            assert_eq!(want, got);
        }
    }
}
