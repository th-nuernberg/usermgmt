pub mod io_util {
    use log::debug;
    use std::collections::HashSet;
    use std::io;

    pub fn user_input() -> String {
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");

        input = input.trim().to_string();
        input
    }

    pub fn hashset_from_vec_str(data: &[String]) -> HashSet<&str> {
        // HashSet::from_iter(data.iter().cloned())
        data.iter().map(|s| s.as_str()).collect::<HashSet<&str>>()
    }

    pub fn get_new_uid(uids: Vec<i32>, group: crate::Group) -> Option<i32> {
        let result;
        // students start at 10000, staff at 1000
        if group == crate::Group::Student {
            result = uids.into_iter().filter(|&i| i >= 10000).collect::<Vec<_>>();
        } else {
            result = uids
                .into_iter()
                .filter(|&i| (1000..10000).contains(&i))
                .collect::<Vec<_>>();
        }

        let max_value = result.iter().max();
        match max_value {
            Some(max) => {
                debug!("Next available uid is: {}", max + 1);
                Some(max + 1)
            }
            None => None,
        }
    }
}
