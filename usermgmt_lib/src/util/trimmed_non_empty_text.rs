use std::borrow::Borrow;

use anyhow::anyhow;

use crate::prelude::AppError;
use derive_more::{AsRef, Display, Into};
/// Contains text which trimmed and is not empty or only white spaces
#[derive(Clone, Debug, Display, AsRef, Into, PartialEq, Eq, Hash)]
pub struct TrimmedNonEmptyText(String);

impl TryFrom<String> for TrimmedNonEmptyText {
    type Error = AppError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let to_validate = value.trim().to_string();

        if to_validate.is_empty() {
            Err(anyhow!("Must not be empty or only white spaces"))
        } else {
            Ok(Self(to_validate))
        }
    }
}

impl Borrow<str> for TrimmedNonEmptyText {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

impl TryFrom<&str> for TrimmedNonEmptyText {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.to_string().try_into()
    }
}

impl TrimmedNonEmptyText {
    pub fn to_lowercase(self) -> Self {
        Self(self.0.to_lowercase())
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    #[test]
    fn error_from_empty_and_only_whitespace() {
        assert!(TrimmedNonEmptyText::try_from("").is_err());
        assert!(TrimmedNonEmptyText::try_from("   ").is_err());
    }
    #[test]
    fn ok_and_trimmed_on_non_empty() {
        assert_eq!(
            TrimmedNonEmptyText::try_from("aaa").unwrap().as_ref(),
            "aaa"
        );
        assert_eq!(
            TrimmedNonEmptyText::try_from("  aaa ").unwrap().as_ref(),
            "aaa"
        );
    }
}
