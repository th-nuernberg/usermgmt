use std::fmt::Display;

use anyhow::bail;

use crate::prelude::AppError;

/// Contains text which trimmed and is not empty/only white spaces
#[derive(Clone, Debug)]
pub struct TrimmedNonEmptyText(String);

impl TryFrom<&str> for TrimmedNonEmptyText {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let to_validate = value.trim().to_string();

        if to_validate.is_empty() {
            bail!("Must not be empty or only white spaces");
        } else {
            Ok(Self(to_validate))
        }
    }
}

impl TrimmedNonEmptyText {
    pub fn to_lowercase(self) -> Self {
        Self(self.0.to_lowercase())
    }
}

impl Display for TrimmedNonEmptyText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TrimmedNonEmptyText {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<TrimmedNonEmptyText> for String {
    fn from(value: TrimmedNonEmptyText) -> Self {
        value.0
    }
}
