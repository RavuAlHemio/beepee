use std::error::Error;
use std::fmt;

use num_rational::Rational32;


#[derive(Debug)]
pub(crate) enum ParseRationalError {
    MoreThanOneDot(usize, usize),
    UnexpectedCharacter(usize, char),
    ErrorParsingMantissa(std::num::ParseIntError),
    DenominatorTooLarge,
}
impl fmt::Display for ParseRationalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MoreThanOneDot(first, second)
                => write!(f, "more than one dot found (at index {} and at index {})", first, second),
            Self::UnexpectedCharacter(pos, c)
                => write!(f, "unexpected character {:?} at position {}", c, pos),
            Self::ErrorParsingMantissa(e)
                => write!(f, "error parsing mantissa: {}", e),
            Self::DenominatorTooLarge
                => write!(f, "denominator too large"),
        }
    }
}
impl Error for ParseRationalError {
}


pub(crate) fn r32_from_decimal(decimal_str: &str) -> Result<Rational32, ParseRationalError> {
    let decimal_chars: Vec<char> = decimal_str.chars().collect();
    let mut dot_index: Option<usize> = None;

    for (i, c) in decimal_chars.iter().enumerate() {
        if i == 0 && *c == '-' {
            // leading minus is OK; it is processed as part of the mantissa
        } else if *c == '.' {
            if let Some(di) = dot_index {
                return Err(ParseRationalError::MoreThanOneDot(di, i));
            }
            dot_index = Some(i);
        } else if !c.is_ascii_digit() {
            return Err(ParseRationalError::UnexpectedCharacter(i, *c));
        }
    }

    let mut mantissa_str: String = String::with_capacity(decimal_chars.len());
    for c in &decimal_chars {
        if *c != '.' {
            mantissa_str.push(*c);
        }
    }
    let mantissa: i32 = mantissa_str.parse()
        .map_err(|e| ParseRationalError::ErrorParsingMantissa(e))?;

    if let Some(di) = dot_index {
        let power_of_10 = (decimal_chars.len() - di) - 1;

        let mut divisor: i32 = 1;
        for _ in 0..power_of_10 {
            let res = match divisor.checked_mul(10) {
                Some(r) => r,
                None => return Err(ParseRationalError::DenominatorTooLarge),
            };
            divisor = res;
        }

        Ok(Rational32::new(mantissa, divisor))
    } else {
        Ok(Rational32::new(mantissa, 1))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn test(num: i32, den: i32, strung: &str) {
        assert_eq!(Rational32::new(num, den), r32_from_decimal(strung).unwrap())
    }

    #[test]
    fn r32_from_integer() {
        test(120, 1, "120");
        test(0, 1, "0");
        test(-42, 1, "-42");
    }

    #[test]
    fn r32_from_trailing_dot() {
        test(120, 1, "120.");
        test(0, 1, "0.");
        test(-42, 1, "-42.");
    }

    #[test]
    fn r32_from_initial_dot() {
        test(3, 25, ".120");
        test(0, 1, ".0");
        test(-21, 50, "-.42");
    }

    #[test]
    fn r32_from_mid_dot() {
        test(6, 5, "1.20");
        test(0, 1, "0.0");
        test(-21, 5, "-4.2");
    }
}
