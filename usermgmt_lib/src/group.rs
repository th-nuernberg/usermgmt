use std::collections::HashMap;

use crate::AppError;
use anyhow::anyhow;

use derive_more::Display;
#[derive(Clone, PartialEq, Copy, Debug, Eq, Display)]
pub enum Group {
    #[display(fmt = "staff")]
    Staff,
    #[display(fmt = "student")]
    Student,
    #[display(fmt = "faculty")]
    Faculty,
}

impl Default for Group {
    fn default() -> Self {
        Self::Student
    }
}

use once_cell::sync::Lazy;

static GROUP_FROM_STR_MAP: Lazy<HashMap<Box<str>, Group>> = Lazy::new(|| {
    let keys = [Group::Staff, Group::Student, Group::Faculty];
    let mut map = HashMap::with_capacity(keys.len() * 2);
    for next in keys {
        let next_lower_case = next.to_string();
        let title_case = {
            let mut iter = next_lower_case.chars();
            iter.next()
                .expect("Every group name must not be an empty string")
                .to_uppercase()
                .collect::<String>()
                + iter.as_str()
        };
        map.insert(next_lower_case.into(), next);
        map.insert(title_case.into(), next);
    }
    map
});

impl std::str::FromStr for Group {
    type Err = AppError;
    fn from_str(input: &str) -> Result<Group, Self::Err> {
        GROUP_FROM_STR_MAP
            .get(input)
            .copied()
            .ok_or_else(|| anyhow!("given group name ({}) is not valid", input))
    }
}
#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn returns_group_from_str() {
        fn assert_case(input: &str, expected: Group) {
            let actual: Group = input.parse().unwrap();
            assert_eq!(expected, actual, "Input: {}", input);
        }

        assert_case("Staff", Group::Staff);
        assert_case("student", Group::Student);
        assert_case("faculty", Group::Faculty);
        assert_case("Faculty", Group::Faculty);
    }
}
