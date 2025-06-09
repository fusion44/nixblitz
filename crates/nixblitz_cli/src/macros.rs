/// Macro to generate a newtype wrapper around an enum and implement `clap::ValueEnum`.
///
/// This macro helps reduce boilerplate when using an enum with clap's value parsing.
///
/// # Arguments
///
/// * `$base_enum:ident`: The identifier of the existing enum (e.g., `SupportedApps`).
/// * `$value_enum_struct:ident`: The identifier for the new wrapper struct to be generated
///   (e.g., `SupportedAppsValueEnum`).
/// * `[$($variant:ident),+ $(,)?]`: A list of variants from the `$base_enum`.
///   These must match the variant names in your `$base_enum` definition.
///
/// # How it works
///
/// The macro generates:
/// 1. A struct `$value_enum_struct` that wraps `$base_enum`.
/// 2. An implementation of `clap::ValueEnum` for `$value_enum_struct`.
///    - `value_variants()`: Provides all variants of the wrapper.
///    - `to_possible_value()`: Converts a variant to its string representation (lowercase).
///    - `from_str()`: Parses a string (case-insensitive or sensitive) into a variant.
/// 3. An implementation of `std::fmt::Display` for `$value_enum_struct` to show the
///    lowercase string representation.
///
/// # Important Note on Updates
/// If you add a new variant to your `$base_enum` (e.g., `SupportedApps`),
/// you **must** also add that variant to the list provided in the macro invocation
/// for it to be included in the `clap::ValueEnum` implementation.
#[macro_export]
macro_rules! define_clap_apps_value_enum {
    (
        $base_enum:ident,
        $value_enum_struct:ident,
        [$($variant:ident),+ $(,)?]
    ) => {
        #[derive(Debug, Clone)]
        pub struct $value_enum_struct($base_enum);

        impl $value_enum_struct {
            /// Allows conversion from the base enum type.
            #[allow(dead_code)] // Suppress dead_code warning if not used by consumer
            pub fn from_base(app: $base_enum) -> Self {
                Self(app)
            }

            /// Allows access to the inner base enum.
            #[allow(dead_code)] // Suppress dead_code warning if not used by consumer
            pub fn inner(&self) -> $base_enum {
                self.0 // Assuming $base_enum is Copy. If not, return &self.0
            }
        }

        impl ValueEnum for $value_enum_struct {
            fn value_variants<'a>() -> &'a [Self] {
                // This static array holds all the instances of our wrapper struct.
                // It's created at compile time.
                // Changed `Self(...)` to `$value_enum_struct(...)` to avoid `self_constructor_from_outer_item` lint.
                const VARIANTS: &'static [$value_enum_struct] = &[
                    $($value_enum_struct($base_enum::$variant)),+
                ];
                VARIANTS
            }

            fn to_possible_value(&self) -> Option<PossibleValue> {
                // Converts the enum variant to its lowercase string representation
                // for clap's help messages and suggestions.
                Some(match self.0 {
                    $(
                        $base_enum::$variant => {
                            PossibleValue::new(stringify!($variant).to_lowercase())
                        }
                    ),+
                })
            }

            fn from_str(input: &str, ignore_case: bool) -> Result<Self, String> {
                // Determines the string to match against, based on ignore_case.
                let normalized_input = if ignore_case {
                    input.to_lowercase()
                } else {
                    input.to_string() // Use the input as is if case matters.
                };

                // Iterate through the known variants and compare their lowercase string
                // representation with the (potentially normalized) input.
                $(
                    if normalized_input == stringify!($variant).to_lowercase() {
                        return Ok(Self($base_enum::$variant));
                    }
                )+

                // If no match is found, construct an informative error message.
                let possible_values_str = Self::value_variants()
                    .iter()
                    .filter_map(|v| v.to_possible_value())
                    .map(|pv| pv.get_name().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                Err(format!(
                    "Invalid variant: '{}'. Possible values are: {}.",
                    input,
                    possible_values_str
                ))
            }
        }

        // Implement Display to show the lowercase string representation.
        // This is useful for printing and consistency.
        impl fmt::Display for $value_enum_struct {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.0 {
                    $(
                        $base_enum::$variant => write!(f, "{}", stringify!($variant).to_lowercase()),
                    )+
                }
            }
        }
    };
}
