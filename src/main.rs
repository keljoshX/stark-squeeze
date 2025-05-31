use stark_squeeze::cli;
use clap::{Parser, Subcommand};
use std::collections::HashSet;
use thiserror::Error;

const APP_NAME: &str = "StarkSqueeze CLI";
const APP_ABOUT: &str = "Interact with StarkSqueeze";
/// CLI arguments for StarkSqueeze
#[derive(Parser, Debug)]
#[command(name = APP_NAME, about = APP_ABOUT)]
struct CliArgs {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Commands for the StarkSqueeze CLI
#[derive(Subcommand, Debug)]
enum Commands {
    /// Upload data to StarkNet
    Upload,
    /// Retrieve data from StarkNet
    Retrieve,
    /// List all uploaded data
    List,
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    match args.command {
        Some(Commands::Upload) => cli::upload_data_cli().await,
        Some(Commands::Retrieve) => cli::retrieve_data_cli().await,
        Some(Commands::List) => cli::list_all_uploads().await,
        None => cli::main_menu().await,
    }
}

#[derive(Debug, Error)]
pub enum DictionaryValidationError {
    #[error("Field contains invalid ASCII characters: {0}")]
    InvalidASCIIError(String),

    #[error("Field has incorrect length (must be 5): {0}")]
    LengthMismatchError(String),

    #[error("Duplicate entry found: {0}")]
    DuplicateEntryError(String),

    #[error("Dictionary missing ASCII characters: {0:?}")]
    MissingCharsError(Vec<char>),
}

pub fn validate_ascii_dictionary(dict_array: &[String]) -> Result<(), DictionaryValidationError> {
    let mut seen = HashSet::new();
    let mut all_chars = HashSet::new();

    for field in dict_array {
        // Length check
        if field.len() != 5 {
            return Err(DictionaryValidationError::LengthMismatchError(field.clone()));
        }

        for ch in field.chars() {
            // ASCII check
            if !(0..=126).contains(&(ch as u8)) {
                return Err(DictionaryValidationError::InvalidASCIIError(field.clone()));
            }
            all_chars.insert(ch);
        }

        // Duplicate check
        if !seen.insert(field.clone()) {
            return Err(DictionaryValidationError::DuplicateEntryError(field.clone()));
        }
    }

    // Coverage check
    let expected_chars: HashSet<char> = (0..=126).map(|c| c as u8 as char).collect();
    let missing: Vec<char> = expected_chars.difference(&all_chars).cloned().collect();

    if !missing.is_empty() {
        return Err(DictionaryValidationError::MissingCharsError(missing));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_valid_dict() -> Vec<String> {
        let chars: Vec<char> = (0..=126).map(|c| c as u8 as char).collect();
        chars
            .chunks(5)
            .map(|chunk| chunk.iter().collect())
            .collect()
    }

    #[test]
    fn test_valid_dictionary() {
        let dict = make_valid_dict();
        assert!(validate_ascii_dictionary(&dict).is_ok());
    }

    #[test]
    fn test_invalid_ascii() {
        let mut dict = make_valid_dict();
        dict[0] = "abce".to_string();
        let result = validate_ascii_dictionary(&dict);
        assert!(matches!(result, Err(DictionaryValidationError::InvalidASCIIError(_))));
    }

    #[test]
    fn test_length_mismatch() {
        let mut dict = make_valid_dict();
        dict[0] = "abcd".to_string(); // only 4 characters
        let result = validate_ascii_dictionary(&dict);
        assert!(matches!(result, Err(DictionaryValidationError::LengthMismatchError(_))));
    }

    #[test]
    fn test_duplicate_entry() {
        let mut dict = make_valid_dict();
        dict[1] = dict[0].clone();
        let result = validate_ascii_dictionary(&dict);
        assert!(matches!(result, Err(DictionaryValidationError::DuplicateEntryError(_))));
    }

    #[test]
    fn test_missing_characters() {
        let mut dict = make_valid_dict();
        dict.pop(); // remove one field => lose 5 characters
        let result = validate_ascii_dictionary(&dict);
        assert!(matches!(result, Err(DictionaryValidationError::MissingCharsError(_))));
    }
}
