use std::collections::HashMap;

use ldap3::{SearchEntry, SearchResult};
use log::warn;

/// Table with depth 3:
/// 1st: Rows of table
/// 2st: columns of a row
/// 3st: cells of a column
type LDAPTableBody = Vec<Vec<Vec<String>>>;
type LDAPHeaders = Vec<String>;
#[derive(Debug, Default, Clone)]
pub struct LdapSearchResult {
    header: LDAPHeaders,
    fields: LDAPTableBody,
}

impl From<LdapSearchResult> for (LDAPHeaders, LDAPTableBody) {
    fn from(value: LdapSearchResult) -> Self {
        (value.header, value.fields)
    }
}

impl LdapSearchResult {
    pub fn new<T>(
        header: impl IntoIterator<Item = T>,
        raw_fields: impl IntoIterator<Item = HashMap<String, Vec<String>>>,
    ) -> Self
    where
        T: ToString,
    {
        let header: Vec<String> = header.into_iter().map(|to| to.to_string()).collect();
        let header_map: HashMap<&str, usize> = header
            .iter()
            .enumerate()
            .map(|(index, key)| (key.as_str(), index))
            .collect();

        let mut fields: Vec<Vec<Vec<String>>> = Default::default();

        let rows: Vec<HashMap<String, Vec<String>>> = raw_fields.into_iter().collect();
        for row_to_convert in rows.into_iter() {
            let mut row: Vec<Vec<String>> = vec![Default::default(); header.len()];
            for (cell_name, cell_values) in row_to_convert.iter() {
                if let Some(&index) = header_map.get(cell_name.as_str()) {
                    let append_to = row.get_mut(index).expect(
                        r#"Index comes from header_map. 
                        Header map is supposed to yield valid indexes for rows"#,
                    );
                    append_to.extend_from_slice(cell_values.as_slice());
                } else {
                    warn!(
                        "Ldap search returned values for the not specified name {}.
                    These values are not included in the table.",
                        cell_name
                    );
                }
            }

            fields.push(row);
        }

        let header: Vec<String> = header
            .into_iter()
            .map(|to_string| to_string.to_string())
            .collect();
        Self { header, fields }
    }

    pub fn from_ldap_raw_search<T>(
        header: impl IntoIterator<Item = T>,
        search_result: &SearchResult,
    ) -> Self
    where
        T: ToString,
    {
        let map = search_result.0.iter().map(|row| {
            let search_entry = SearchEntry::construct(row.to_owned());
            search_entry.attrs
        });
        Self::new(header, map)
    }

    pub fn headers(&self) -> Vec<&str> {
        self.header.iter().map(|string| string.as_str()).collect()
    }

    pub fn fields(&self) -> Vec<Vec<Vec<&str>>> {
        self.fields
            .as_slice()
            .iter()
            .map(|string| {
                string
                    .iter()
                    .map(|string| string.iter().map(|to| to.as_str()).collect())
                    .collect()
            })
            .collect()
    }
}
