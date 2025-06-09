use core::fmt;
use std::str::FromStr;

use strum_macros::{EnumCount as EnumCountMacro, EnumIter, VariantArray};

/// Represents the default shell to use for the system
#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter, EnumCountMacro, VariantArray)]
pub enum Shell {
    /// Use Bash as the default shell
    Bash,
    /// Use Nushell as the default shell
    Nushell,
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shell::Bash => write!(f, "Bash"),
            Shell::Nushell => write!(f, "Nushell"),
        }
    }
}

impl Shell {
    pub fn to_nix_package_name(&self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Nushell => "nushell",
        }
    }
}

impl FromStr for Shell {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Bash" | "bash" => Ok(Shell::Bash),
            "Nushell" | "nushell" => Ok(Shell::Nushell),
            _ => Err(format!(
                "Failed to parse '{}' as Shell. Expected one of (case-insensitive): [\"Bash\", \"Nushell\"].",
                s
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::{EnumCount, IntoEnumIterator};

    #[test]
    fn test_display_format() {
        assert_eq!(Shell::Bash.to_string(), "Bash");
        assert_eq!(Shell::Nushell.to_string(), "Nushell");
    }

    #[test]
    fn test_from_str_trait() {
        assert_eq!("Bash".parse::<Shell>(), Ok(Shell::Bash));
        assert_eq!("nushell".parse::<Shell>(), Ok(Shell::Nushell));

        let parse_result = "invalid".parse::<Shell>();
        assert!(parse_result.is_err());
        if let Err(e) = parse_result {
            assert!(e.contains("Failed to parse 'invalid' as Shell"));
        }
    }

    #[test]
    fn test_enum_iter_and_count() {
        assert_eq!(Shell::iter().count(), Shell::COUNT);
        assert_eq!(Shell::COUNT, 2); // Explicitly check count if it's stable
        let mut found_nushell = false;
        for platform in Shell::iter() {
            if platform == Shell::Nushell {
                found_nushell = true;
            }
        }
        assert!(found_nushell, "EnumIter did not yield Nushell variant");
    }
}
