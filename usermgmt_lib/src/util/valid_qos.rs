use derive_more::{AsRef, Display, Into, IntoIterator};

use crate::{config::MgmtConfig, prelude::AppResult, Group};

use super::TrimmedNonEmptyText;
#[derive(Debug, Display, Into, Clone, AsRef, PartialEq, Eq)]
pub struct ValidQos(String);

impl ValidQos {
    /// # Errors
    ///
    /// - If parameter `value` is not a valid non-empty text. See [`TrimmedNonEmptyText`]
    /// - If parameter `value` is not listed within the given valid quality of services aka parameter `valid_qos`
    pub fn new(value: String, valid_qos: &[impl AsRef<str>]) -> AppResult<Self> {
        let trimmed_not_empty: TrimmedNonEmptyText = value.try_into()?;
        if valid_qos
            .iter()
            .any(|to_check_against| to_check_against.as_ref() == trimmed_not_empty.as_ref())
        {
            Ok(Self(trimmed_not_empty.into()))
        } else {
            Err(anyhow::anyhow!(
                "Given qos {} is none of the valid qoses",
                trimmed_not_empty
            ))
        }
    }

    pub fn default_qos_from_conf(group: Group, config: &MgmtConfig) -> Self {
        let value = match group {
            Group::Staff | Group::Faculty => config.staff_default_qos.clone(),
            Group::Student => config.student_default_qos.clone(),
        };
        Self(value)
    }
}

#[derive(Debug, Clone, IntoIterator)]
#[into_iterator(ref)]
pub struct ValidGroupOfQos(Vec<ValidQos>);

impl ValidGroupOfQos {
    pub fn new(value: Vec<TrimmedNonEmptyText>, valid_qos: &[impl AsRef<str>]) -> AppResult<Self> {
        let content = value
            .into_iter()
            .map(|from| ValidQos::new(from.into(), valid_qos))
            .collect::<AppResult<_>>()?;

        Ok(Self(content))
    }

    pub fn from_group(group: Group, config: &MgmtConfig) -> AppResult<Self> {
        let from_config = match group {
            Group::Staff => config.staff_qos.clone(),
            Group::Student => config.student_qos.clone(),
            Group::Faculty => config.staff_qos.clone(),
        }
        .into_iter()
        .map(|to_convert| ValidQos::new(to_convert, &config.valid_qos))
        .collect::<AppResult<_>>()?;

        Ok(Self(from_config))
    }

    pub fn contains_other_qos(&self, other: &ValidQos) -> bool {
        self.0.iter().any(|next| other == next)
    }
}

impl From<ValidGroupOfQos> for Vec<String> {
    fn from(value: ValidGroupOfQos) -> Self {
        value.0.into_iter().map(|qos| qos.into()).collect()
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    #[test]
    fn error_if_not_valid_qos() {
        let valid_qos: Vec<&str> = vec!["basic", "normal"];
        assert!(ValidQos::new("bas".into(), &valid_qos).is_err());
    }
    #[test]
    fn error_if_not_valid_qos_in_any_number_of_qos() {
        let valid_qos: Vec<&str> = vec!["basic", "normal"];
        assert!(ValidGroupOfQos::new(
            vec!["basic".try_into().unwrap(), "basi".try_into().unwrap()],
            &valid_qos
        )
        .is_err());
    }
    #[test]
    fn ok_if_valid_qos() {
        let valid_qos: Vec<&str> = vec!["basic", "normal"];
        assert_eq!(
            ValidQos::new("basic".into(), &valid_qos).unwrap().as_ref(),
            "basic"
        );
    }
    #[test]
    fn ok_if_valid_group_of_qos() {
        let valid_qos: Vec<&str> = vec!["basic", "normal", "init"];
        assert!(ValidGroupOfQos::new(
            vec!["basic".try_into().unwrap(), "normal".try_into().unwrap()],
            &valid_qos
        )
        .is_ok());
    }
}
