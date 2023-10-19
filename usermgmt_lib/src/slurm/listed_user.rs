/// Character which splits cells in a row
/// from the output given back by sacctmgr command
/// with the cli option --parseable
const SPLIT_BETWEEN_CELLS: char = '|';

#[derive(Debug, Default)]
pub struct ListedUser {
    headers: Vec<String>,
    fields: Vec<Vec<String>>,
}

impl ListedUser {
    /// Turns parse able text from sacctmgr into a rust struct which
    /// can be queried for the headers and fields for slurm query much better.
    /// It assumes that all values in given text are separated by the char '|'
    pub fn new(input: &str) -> Option<Self> {
        let mut lines = input.lines();
        let headers = get_row(lines.next()?);

        let mut fields: Vec<Vec<String>> = Default::default();
        for next_line in lines {
            let row: Vec<String> = get_row(next_line);
            fields.push(row);
        }
        return Some(Self { headers, fields });
        fn get_row(line: &str) -> Vec<String> {
            let mut row: Vec<String> = line
                .trim()
                .split(SPLIT_BETWEEN_CELLS)
                .map(|string_slice| string_slice.to_string())
                .collect();

            _ = row.pop();
            row
        }
    }
    pub fn fields(&self) -> impl Iterator<Item = &[String]> {
        self.fields.iter().map(|row| row.as_slice())
    }
    pub fn headers(&self) -> &[String] {
        self.headers.as_slice()
    }
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn parses_correctly() {
        let input = "User|Account|Def QOS|QOS|
|root||normal|
dev_user|root||normal|
root|root||normal|
|thn||normal|
|cs||normal|";

        let actual = ListedUser::new(input);
        insta::assert_debug_snapshot!(actual);
    }
}
