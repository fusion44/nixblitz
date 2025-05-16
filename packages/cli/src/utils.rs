use nixblitzlib::strings::{Strings, STRINGS};

use crate::errors::CliError;

pub trait GetStringOrCliError {
    fn get_or_err(&self) -> Result<&str, CliError>;
}

impl GetStringOrCliError for Strings {
    fn get_or_err(&self) -> Result<&str, CliError> {
        match self {
            Strings::PasswordInputPlaceholderMain => Ok(STRINGS
                .get(self)
                .ok_or(CliError::StringRetrievalError(self.to_string()))?),
            Strings::PasswordInputPlaceholderConfirm => Ok(STRINGS
                .get(self)
                .ok_or(CliError::StringRetrievalError(self.to_string()))?),
        }
    }
}

/// Splits a string slice into parts based on a list of byte indices.
///
/// The function treats the provided indices as points at which to split
/// the string. It includes the segment before the first index, segments
/// between consecutive indices, and the segment after the last index.
///
/// The input `indexes` slice is sorted internally to ensure correct order.
/// Duplicate indices are effectively ignored. Indices that are out of bounds
/// or do not fall on UTF-8 character boundaries are skipped, and warnings
/// are printed to stderr.
///
/// # Arguments
///
/// * `title` - The string slice (`&str`) to split.
/// * `indexes` - A slice of `usize` representing the byte indices at which
///   to split the string. Order doesn't matter; it will be sorted.
///
/// # Returns
///
/// A `Vec<String>` containing the parts of the string after splitting.
/// Returns a vector with the original string if `indexes` is empty.
/// Returns an empty vector if the `title` is empty.
///
/// # Examples
///
/// ```
/// let title = "[1] Apps";
/// // Byte indices: (=0, 1=1, )=2, ' '=3, A=4, p=5, p=6, s=7. Len=8
/// let indexes = vec![1, 4]; // Split points
/// // Expected parts: "(", "1) ", "Apps"
/// let parts = extract_string_parts(title, &indexes);
/// assert_eq!(parts, vec!["[".to_string(), "1] ".to_string(), "Apps".to_string()]);
/// ```
///
/// ```
/// let title = "Hello";
/// let indexes = vec![]; // No split points
/// let parts = extract_string_parts(title, &indexes);
/// assert_eq!(parts, vec!["Hello".to_string()]);
/// ```
pub fn extract_string_parts(title: &str, indexes: &[usize]) -> Vec<String> {
    // --- Function Implementation ---
    let mut parts: Vec<String> = Vec::new();
    let title_len = title.len();

    if title.is_empty() {
        return parts; // Return empty vec for empty title
    }

    // --- Preprocess Indices ---
    // 1. Filter out invalid indices (out of bounds or not char boundaries)
    // 2. Sort the valid indices
    // 3. Deduplicate
    let mut sorted_valid_indexes: Vec<usize> = indexes
        .iter()
        .filter(|&&idx| {
            if idx > title_len {
                eprintln!(
                    "Warning: Index {} out of bounds (len: {}). Skipping.",
                    idx, title_len
                );
                false
            } else if !title.is_char_boundary(idx) {
                eprintln!(
                    "Warning: Index {} is not a character boundary. Skipping.",
                    idx
                );
                false
            } else {
                true // Keep valid indices
            }
        })
        .cloned() // Convert `&&usize` to `usize`
        .collect();

    sorted_valid_indexes.sort_unstable(); // Sort the valid indices
    sorted_valid_indexes.dedup(); // Remove duplicates

    // --- Extract Parts ---
    let mut last_index = 0;

    // Iterate through the sorted, unique, valid split points
    for &current_index in &sorted_valid_indexes {
        // Extract the part from the last split point up to the current one
        if current_index > last_index {
            // Ensure we don't create empty slices if indices are consecutive
            // Slicing here is safe because both last_index (initially 0 or a previous valid index)
            // and current_index have been validated as character boundaries.
            match title.get(last_index..current_index) {
                Some(part_slice) => parts.push(part_slice.to_string()),
                None => {
                    // This should ideally not happen due to prior checks, but good to handle defensively
                    eprintln!(
                        "Error: Failed to slice between {} and {}",
                        last_index, current_index
                    );
                }
            }
        }
        last_index = current_index; // Move the last split point forward
    }

    // Add the final part from the last split point to the end of the string
    if last_index < title_len {
        // Slicing here is safe because last_index is a validated boundary
        // and title_len is always a valid boundary.
        match title.get(last_index..title_len) {
            Some(part_slice) => parts.push(part_slice.to_string()),
            None => {
                eprintln!(
                    "Error: Failed to slice between {} and {}",
                    last_index, title_len
                );
            }
        }
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ascii_1() {
        let title = "[1] Apps"; // Len=8
                                // Split points: 1 (after '('), 4 (after ' ')
        let indexes = vec![1, 4];
        // Expected: "(", "1) ", "Apps"
        let expected = vec!["[".to_string(), "1] ".to_string(), "Apps".to_string()];
        assert_eq!(
            extract_string_parts(title, &indexes),
            expected,
            "Test Case: Basic ASCII 1"
        );
    }

    #[test]
    fn test_basic_ascii_2() {
        let title = "Hello Rust World!"; // Len=17
                                         // Split points: 6 (after ' '), 11 (after ' ')
        let indexes = vec![6, 11];
        // Expected: "Hello ", "Rust ", "World!"
        let expected = vec![
            "Hello ".to_string(),
            "Rust ".to_string(),
            "World!".to_string(),
        ];
        assert_eq!(
            extract_string_parts(title, &indexes),
            expected,
            "Test Case: Basic ASCII Original"
        );
    }

    #[test]
    fn test_utf8_multibyte() {
        let title = "你好世界"; // ni hao shi jie
                                // Byte indices: 你=0(3b), 好=3(3b), 世=6(3b), 界=9(3b). Len = 12
                                // Split point: 6 (between 好 and 世)
        let indexes = vec![6];
        // Expected: "你好", "世界"
        let expected = vec!["你好".to_string(), "世界".to_string()];
        assert_eq!(
            extract_string_parts(title, &indexes),
            expected,
            "Test Case: UTF-8 Multibyte"
        );
    }

    #[test]
    fn test_utf8_mixed() {
        let title = "Go语言_你好"; // Go Language _ ni hao
                                   // Byte indices: G=0, o=1, 语=2(3b), 言=5(3b), _=8, 你=9(3b), 好=12(3b). Len = 15
                                   // Split points: 2 (before 语), 8 (before _), 9 (before 你)
        let indexes = vec![2, 8, 9];
        // Expected: "Go", "语言", "_", "你好"
        let expected = vec![
            "Go".to_string(),
            "语言".to_string(),
            "_".to_string(),
            "你好".to_string(),
        ];
        assert_eq!(
            extract_string_parts(title, &indexes),
            expected,
            "Test Case: UTF-8 Mixed"
        );
    }

    #[test]
    fn test_empty_title() {
        let title = "";
        let indexes = vec![0]; // Index 0 is valid but doesn't split anything
        let expected: Vec<String> = vec![];
        assert_eq!(
            extract_string_parts(title, &indexes),
            expected,
            "Test Case: Empty Title with Index 0"
        );

        let indexes_also_empty = vec![];
        assert_eq!(
            extract_string_parts(title, &indexes_also_empty),
            expected,
            "Test Case: Empty Title, Empty Indexes"
        );
    }

    #[test]
    fn test_empty_indexes() {
        let title = "Some Title";
        let indexes = vec![]; // No split points, should return the whole string
        let expected: Vec<String> = vec![title.to_string()];
        assert_eq!(
            extract_string_parts(title, &indexes),
            expected,
            "Test Case: Empty Indexes"
        );
    }

    #[test]
    fn test_single_split_point() {
        let title = "FirstSecond"; // Len = 11
                                   // Split point: 5 (between 't' and 'S')
        let indexes = vec![5];
        let expected = vec!["First".to_string(), "Second".to_string()];
        assert_eq!(
            extract_string_parts(title, &indexes),
            expected,
            "Test Case: Single Split Point"
        );
    }

    #[test]
    fn test_split_at_start() {
        let title = "Test"; // Len = 4
                            // Split point: 0
        let indexes = vec![0];
        // Expected: "" (before 0), "Test" (after 0) -> filter out empty start?
        // Current logic: last_index=0. current_index=0. Skip first loop. last_index=0. Add title[0..4].
        let expected = vec!["Test".to_string()]; // The empty part before index 0 is implicitly skipped
        assert_eq!(
            extract_string_parts(title, &indexes),
            expected,
            "Test Case: Split at Start"
        );
    }

    #[test]
    fn test_split_at_end() {
        let title = "Test"; // Len = 4
                            // Split point: 4
        let indexes = vec![4];
        // Expected: "Test" (before 4), "" (after 4) -> filter out empty end?
        // Current logic: last_index=0. current_index=4. Add title[0..4]. last_index=4. Skip final part.
        let expected = vec!["Test".to_string()]; // The empty part after index 4 is implicitly skipped
        assert_eq!(
            extract_string_parts(title, &indexes),
            expected,
            "Test Case: Split at End"
        );
    }

    #[test]
    fn test_split_at_start_and_end() {
        let title = "Test"; // Len = 4
        let indexes = vec![0, 4];
        // Expected: "" (before 0), "Test" (between 0 and 4), "" (after 4)
        // Current logic: last=0, curr=0 -> skip. last=0. curr=4 -> add "Test". last=4. Skip final part.
        let expected = vec!["Test".to_string()];
        assert_eq!(
            extract_string_parts(title, &indexes),
            expected,
            "Test Case: Split at Start and End"
        );
    }

    #[test]
    fn test_consecutive_split_points() {
        let title = "ABCD"; // Len = 4
                            // Split points: 1, 2, 3
        let indexes = vec![1, 2, 3];
        // Expected: "A", "B", "C", "D"
        let expected = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ];
        assert_eq!(
            extract_string_parts(title, &indexes),
            expected,
            "Test Case: Consecutive Split Points"
        );
    }

    #[test]
    fn test_duplicate_split_points() {
        let title = "NoSplit"; // Len = 7
                               // Split points: 2, 2 (duplicates handled)
        let indexes = vec![2, 2];
        // Expected: "No", "Split"
        let expected = vec!["No".to_string(), "Split".to_string()];
        assert_eq!(
            extract_string_parts(title, &indexes),
            expected,
            "Test Case: Duplicate Split Points"
        );
    }

    #[test]
    fn test_index_out_of_bounds_ignored() {
        let title = "Bound"; // Len = 5
                             // Split points: 2, 6 (6 is ignored)
        let indexes = vec![2, 6];
        // Expected: "Bo", "und"
        let expected = vec!["Bo".to_string(), "und".to_string()];
        // Warning will be printed to stderr for index 6 during test execution
        assert_eq!(
            extract_string_parts(title, &indexes),
            expected,
            "Test Case: Index Out Of Bounds Ignored"
        );
    }

    #[test]
    fn test_index_not_char_boundary_ignored() {
        let title = "你好世界"; // Len = 12
                                // Split points: 4 (invalid), 6 (valid)
        let indexes = vec![4, 6];
        // Expected: "你好", "世界" (split only happens at valid index 6)
        let expected = vec!["你好".to_string(), "世界".to_string()];
        // Warning will be printed to stderr for index 4 during test execution
        assert_eq!(
            extract_string_parts(title, &indexes),
            expected,
            "Test Case: Index Not Char Boundary Ignored"
        );
    }

    #[test]
    fn test_unsorted_indexes_handled() {
        let title = "Hello Rust World!"; // Len 17
                                         // Split points: 11, 6 (unsorted) -> sorted to [6, 11]
        let indexes = vec![11, 6];
        // Expected: "Hello ", "Rust ", "World!"
        let expected = vec![
            "Hello ".to_string(),
            "Rust ".to_string(),
            "World!".to_string(),
        ];
        assert_eq!(
            extract_string_parts(title, &indexes),
            expected,
            "Test Case: Unsorted Indexes Handled"
        );
    }
}
