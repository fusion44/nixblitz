use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::errors::ParseError;

/// Represents a numerical value that can be an unsigned integer, signed integer, or float.
/// Each variant holds an `Option` to allow for the absence of a value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NumberValue {
    /// Represents unsigned integers with a size of 16 bits, ranging from 0 to 65535.
    U16(Option<u16>),
    /// Represents unsigned integers with the size of a pointer
    /// (usually 64 bits on 64-bit systems and 32 bits on 32-bit systems).
    UInt(Option<usize>),
    /// Represents integers with the size of a pointer
    /// (usually 64 bits on 64-bit systems and 32 bits on 32-bit systems).
    Int(Option<isize>),
    /// A 64-bit floating-point type (specifically, the “binary64”
    /// type defined in IEEE 754-2008).
    Float(Option<f64>),
}

impl NumberValue {
    /// Converts the `NumberValue` to a `String`.
    /// Returns the provided default string if the value is `None`.
    ///
    /// # Arguments
    ///
    /// * `default` - The default string to return if the value is `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nixblitz_core::number_value::NumberValue;
    ///
    /// let num = NumberValue::UInt(Some(42));
    /// assert_eq!(num.to_string_or("default"), "42");
    /// let none_num = NumberValue::UInt(None);
    /// assert_eq!(none_num.to_string_or("default"), "default");
    /// ```
    pub fn to_string_or(&self, default: &str) -> String {
        match self {
            NumberValue::U16(Some(v)) => v.to_string(),
            NumberValue::UInt(Some(v)) => v.to_string(),
            NumberValue::Int(Some(v)) => v.to_string(),
            NumberValue::Float(Some(v)) => v.to_string(),
            _ => default.to_string(),
        }
    }

    /// Parses a `String` into a `NumberValue` of the specified type.
    /// Returns an error if the string cannot be parsed into the desired type.
    ///
    /// # Arguments
    ///
    /// * `input` - The string to parse.
    /// * `into` - The `NumberValue` variant to parse into.
    ///
    /// # Errors
    ///
    /// Returns a `CliError::StringParseError` if parsing fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use nixblitz_core::number_value::NumberValue;
    ///
    /// let result = NumberValue::from_string("42".to_string(), NumberValue::UInt(None));
    /// assert!(result.is_ok());
    /// ```
    pub fn from_string(input: String, into: NumberValue) -> Result<NumberValue, ParseError> {
        match into {
            NumberValue::U16(_) => input
                .parse::<u16>()
                .map(|res| NumberValue::U16(Some(res)))
                .map_err(|_| ParseError::StringParseError(input)),
            NumberValue::UInt(_) => input
                .parse::<usize>()
                .map(|res| NumberValue::UInt(Some(res)))
                .map_err(|_| ParseError::StringParseError(input)),
            NumberValue::Int(_) => input
                .parse::<isize>()
                .map(|res| NumberValue::Int(Some(res)))
                .map_err(|_| ParseError::StringParseError(input)),
            NumberValue::Float(_) => input
                .parse::<f64>()
                .map_err(|_| ParseError::StringParseError(input.clone()))
                .and_then(|res| {
                    if res == f64::INFINITY {
                        Err(ParseError::StringParseError(input))
                    } else {
                        Ok(NumberValue::Float(Some(res)))
                    }
                }),
        }
    }

    /// Checks if the `NumberValue` is a float.
    ///
    /// # Examples
    ///
    /// ```
    /// use nixblitz_core::number_value::NumberValue;
    ///
    /// let num = NumberValue::Float(Some(3.14));
    /// assert!(num.is_float());
    /// ```
    pub fn is_float(&self) -> bool {
        matches!(self, NumberValue::Float(_))
    }

    /// Returns a new `NumberValue` with the same variant but with `None` as its value.
    ///
    /// # Examples
    ///
    /// ```
    /// use nixblitz_core::number_value::NumberValue;
    ///
    /// let num = NumberValue::UInt(Some(42));
    /// let none_num = num.as_none();
    /// assert!(matches!(none_num, NumberValue::UInt(None)));
    /// ```
    pub fn as_none(&self) -> NumberValue {
        match self {
            NumberValue::U16(_) => NumberValue::U16(None),
            NumberValue::UInt(_) => NumberValue::UInt(None),
            NumberValue::Int(_) => NumberValue::Int(None),
            NumberValue::Float(_) => NumberValue::Float(None),
        }
    }

    /// Checks if the `NumberValue` is `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nixblitz_core::number_value::NumberValue;
    ///
    /// let num = NumberValue::UInt(None);
    /// assert!(num.is_none());
    /// let num_with_value = NumberValue::UInt(Some(42));
    /// assert!(!num_with_value.is_none());
    /// ```
    pub fn is_none(&self) -> bool {
        matches!(
            self,
            NumberValue::U16(None)
                | NumberValue::UInt(None)
                | NumberValue::Int(None)
                | NumberValue::Float(None)
        )
    }

    /// Sets the value of the `NumberValue`. The input value must be a float
    /// and will be converted to the underlying enum variant.
    ///
    /// # Arguments
    ///
    /// * `value` - The new value to set.
    ///
    /// # Examples
    ///
    /// ```
    /// use nixblitz_core::number_value::NumberValue;
    ///
    /// let mut num = NumberValue::UInt(None);
    /// num.set_value(Some(42.0));
    /// assert_eq!(num, NumberValue::UInt(Some(42)));
    /// ```
    pub fn set_value(&mut self, value: Option<f64>) {
        match self {
            NumberValue::U16(_) => {
                *self = NumberValue::U16(value.map(|v| v as u16));
            }
            NumberValue::UInt(_) => {
                *self = NumberValue::UInt(value.map(|v| v as usize));
            }
            NumberValue::Int(_) => {
                *self = NumberValue::Int(value.map(|v| v as isize));
            }
            NumberValue::Float(_) => {
                *self = NumberValue::Float(value);
            }
        }
    }
}

impl Display for NumberValue {
    /// Converts the `NumberValue` to a `String`.
    /// Returns an empty string if the value is `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nixblitz_core::number_value::NumberValue;
    ///
    /// let num = NumberValue::UInt(Some(42));
    /// assert_eq!(num.to_string(), "42");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberValue::U16(Some(v)) => write!(f, "{}", v),
            NumberValue::UInt(Some(v)) => write!(f, "{}", v),
            NumberValue::Int(Some(v)) => write!(f, "{}", v),
            NumberValue::Float(Some(v)) => write!(f, "{}", v),
            _ => write!(f, ""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        assert_eq!(NumberValue::U16(Some(42)).to_string(), "42");
        assert_eq!(NumberValue::UInt(Some(42)).to_string(), "42");
        assert_eq!(NumberValue::Int(Some(-42)).to_string(), "-42");
        assert_eq!(NumberValue::Float(Some(3.9999)).to_string(), "3.9999");
        assert_eq!(NumberValue::UInt(None).to_string(), "");
        assert_eq!(NumberValue::Int(None).to_string(), "");
        assert_eq!(NumberValue::Float(None).to_string(), "");
    }

    #[test]
    fn test_from_string() {
        assert_eq!(
            NumberValue::from_string("99".to_string(), NumberValue::U16(None)).unwrap(),
            NumberValue::U16(Some(99))
        );
        assert_eq!(
            NumberValue::from_string("42".to_string(), NumberValue::UInt(None)).unwrap(),
            NumberValue::UInt(Some(42))
        );
        assert_eq!(
            NumberValue::from_string("-42".to_string(), NumberValue::Int(None)).unwrap(),
            NumberValue::Int(Some(-42))
        );
        assert_eq!(
            NumberValue::from_string("3.9999".to_string(), NumberValue::Float(None)).unwrap(),
            NumberValue::Float(Some(3.9999))
        );

        assert!(NumberValue::from_string("abc".to_string(), NumberValue::U16(None)).is_err());
        assert!(NumberValue::from_string("abc".to_string(), NumberValue::UInt(None)).is_err());
        assert!(NumberValue::from_string("abc".to_string(), NumberValue::Int(None)).is_err());
        assert!(NumberValue::from_string("abc".to_string(), NumberValue::Float(None)).is_err());
    }

    #[test]
    fn test_is_float() {
        assert!(NumberValue::Float(Some(3.9999)).is_float());
        assert!(!NumberValue::U16(Some(99)).is_float());
        assert!(!NumberValue::UInt(Some(42)).is_float());
        assert!(!NumberValue::Int(Some(-42)).is_float());
    }

    #[test]
    fn test_is_none() {
        assert!(NumberValue::U16(None).is_none());
        assert!(NumberValue::UInt(None).is_none());
        assert!(NumberValue::Int(None).is_none());
        assert!(NumberValue::Float(None).is_none());
        assert!(!NumberValue::U16(Some(99)).is_none());
        assert!(!NumberValue::UInt(Some(42)).is_none());
        assert!(!NumberValue::Int(Some(-42)).is_none());
        assert!(!NumberValue::Float(Some(3.9999)).is_none());
    }

    #[test]
    fn test_as_none() {
        assert_eq!(NumberValue::U16(Some(99)).as_none(), NumberValue::U16(None));
        assert_eq!(
            NumberValue::UInt(Some(42)).as_none(),
            NumberValue::UInt(None)
        );
        assert_eq!(
            NumberValue::Int(Some(-42)).as_none(),
            NumberValue::Int(None)
        );
        assert_eq!(
            NumberValue::Float(Some(3.9999)).as_none(),
            NumberValue::Float(None)
        );
    }

    #[test]
    fn test_from_string_overflow() {
        assert!(NumberValue::from_string("65536".to_string(), NumberValue::U16(None)).is_err());
        assert!(
            NumberValue::from_string("18446744073709551616".to_string(), NumberValue::UInt(None))
                .is_err()
        );
        assert!(
            NumberValue::from_string("9223372036854775808".to_string(), NumberValue::Int(None))
                .is_err()
        );
        assert!(
            NumberValue::from_string(
                "1.7976931348623157e+309".to_string(),
                NumberValue::Float(None)
            )
            .is_err()
        );
    }

    #[test]
    fn test_set_value() {
        let mut num_u16 = NumberValue::U16(None);
        num_u16.set_value(Some(42.0));
        assert_eq!(num_u16, NumberValue::U16(Some(42)));

        let mut num_uint = NumberValue::UInt(None);
        num_uint.set_value(Some(42.0));
        assert_eq!(num_uint, NumberValue::UInt(Some(42)));

        let mut num_int = NumberValue::Int(None);
        num_int.set_value(Some(-42.0));
        assert_eq!(num_int, NumberValue::Int(Some(-42)));

        let mut num_float = NumberValue::Float(None);
        num_float.set_value(Some(3.9999));
        assert_eq!(num_float, NumberValue::Float(Some(3.9999)));

        // Test setting None
        let mut num_none = NumberValue::UInt(Some(42));
        num_none.set_value(None);
        assert_eq!(num_none, NumberValue::UInt(None));
    }
}
