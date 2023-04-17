pub mod io_util {
    use crate::Group;
    use log::debug;
    use std::collections::HashSet;
    use std::io;

    const STUDENT_UID: i32 = 10_000;
    const STAFF_UID: i32 = 1_000;

    pub fn user_input() -> String {
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");

        input = input.trim().to_string();
        input
    }

    pub fn hashset_from_vec_str<R>(data: &'_ [R]) -> HashSet<&'_ str>
    where
        R: AsRef<str>,
    {
        data.iter().map(|s| s.as_ref()).collect::<HashSet<&str>>()
    }

    /// Returns uid which can be used for a new user.
    /// # Errors
    /// - if next uid would cause overflow because of its size
    /// - if next staff uid would be so big that it becomes a student id
    ///  
    /// TODO: replace String as error with type implementing error trait.
    /// TODO: could u32 be better than i32 ?  is there a need for a negative uid.
    pub fn get_new_uid(uids: &[i32], group: crate::Group) -> Result<i32, String> {
        // students start at 10000, staff at 1000
        let result = if group == Group::Student {
            uids.iter()
                .filter(|&&i| i >= STUDENT_UID)
                .collect::<Vec<_>>()
        } else {
            uids.iter()
                .filter(|i| (STAFF_UID..STUDENT_UID).contains(i))
                .collect::<Vec<_>>()
        };

        let max_value = result.iter().max();
        match max_value {
            Some(&&max) => {
                debug!("Next available uid is: {}", max + 1);

                let (next_uid, has_overflow) = max.overflowing_add(1);

                if has_overflow {
                    return Err(
                        "Next uid would cause an overflow for an unsigned integer 32".to_string(),
                    );
                }

                if group == Group::Staff && next_uid >= STUDENT_UID {
                    return Err(format!("Next uid for staff goes into uid range of students !. Students range starts at {}", STUDENT_UID));
                }

                Ok(next_uid)
            }
            None => {
                if group == Group::Student {
                    Ok(STUDENT_UID + 1)
                } else {
                    Ok(STAFF_UID + 1)
                }
            }
        }
    }

    #[cfg(test)]
    mod testing {
        use maplit::hashset;

        use super::*;
        #[test]
        fn should_return_next_uid() {
            // With existing staff and students
            let example_uids = vec![10001, 10002, 10005, 10003, 1001];
            assert_return_next_uid(&vec![], Group::Staff, 1001);
            assert_return_next_uid(&vec![], Group::Student, 10001);
            // Only with existing staff
            assert_return_next_uid(&vec![1001, 1002], Group::Student, 10001);
            // Only with existing students
            assert_return_next_uid(&vec![10001, 10002], Group::Staff, 1001);
            assert_return_next_uid(&example_uids, Group::Student, 10006);
            assert_return_next_uid(&example_uids, Group::Staff, 1002);
        }

        #[test]
        fn should_return_error_for_overflow() {
            let actual = get_new_uid(&vec![i32::MAX], Group::Student);
            assert!(actual.is_err());
        }
        #[test]
        fn should_return_error_for_staff_into_student() {
            let actual = get_new_uid(&vec![STUDENT_UID - 1], Group::Staff);
            assert!(actual.is_err());
        }

        #[test]
        fn should_hashset_from_vec_str() {
            let given = vec!["one", "one", "two", "three"];
            let actual = hashset_from_vec_str(&given);
            let expected = hashset! {"one", "two", "three"};
            assert_eq!(expected, actual);
        }

        fn assert_return_next_uid(uids: &[i32], group: crate::Group, expected_uid: i32) {
            let actual = get_new_uid(uids, group);
            let actual_value = actual.expect("Should not be an error for valid input");
            assert_eq!(actual_value, expected_uid);
        }
    }
}
