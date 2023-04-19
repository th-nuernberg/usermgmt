use std::{borrow::Borrow, collections::HashMap, hash::Hash};

use ldap3::{SearchEntry, SearchResult};
use log::warn;

/// Returns a pretty ASCII table from the result of an LDAP search query.
/// First row is the title row which shows all used field name.
/// Subsequent rows contain values to each field name for an LDAP entity.
///
/// If there is no value for the field, then the cell of in the column is white space only
pub fn ldap_search_to_pretty_table(
    search_by_in_title: &[&str],
    search_result: &SearchResult,
) -> String {
    let to_table = convert_ldap_listing_to_vec_hash_map(search_result);

    contruct_table_from_vec_hash_map(search_by_in_title, &to_table)
}

/// Returns rows. Every row consists comma separated cells. Every cell is key value pair with "="
/// sign in the middle.
///
/// Example for cell: name=example
///
/// If there is no value for the field, then there is nothing from right of the "=" sign
pub fn ldap_simple_output(search_by_in_title: &[&str], search_result: &SearchResult) -> String {
    let to_simple_output = convert_ldap_listing_to_vec_hash_map(search_result);

    contruct_simple_output_from_vec_hash_map(search_by_in_title, &to_simple_output)
}

fn convert_ldap_listing_to_vec_hash_map(
    search_result: &SearchResult,
) -> Vec<HashMap<String, Vec<String>>> {
    search_result
        .0
        .iter()
        .map(|row| {
            let search_entry = SearchEntry::construct(row.to_owned());
            search_entry.attrs
        })
        .collect()
}

pub fn contruct_simple_output_from_vec_hash_map<S>(
    search_by_in_title: &[&str],
    attrs: &[HashMap<S, Vec<S>>],
) -> String
where
    S: Borrow<str> + std::fmt::Display + Eq + Hash,
{
    attrs
        .into_iter()
        .map(|next_row| {
            search_by_in_title
                .iter()
                .map(|&next_field| {
                    if let Some(values) = next_row.get(next_field) {
                        let values = values.join("|");
                        format!("{}={}", next_field, values)
                    } else {
                        format!("{}=", next_field)
                    }
                })
                .collect::<Vec<String>>()
                .join(",")
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// Returns a nice formatted table with title as 1. row form `search_by_in_title` and
/// subsequent rows with the values of the vec to a key
pub fn contruct_table_from_vec_hash_map<S>(
    search_by_in_title: &[&str],
    attrs: &[HashMap<S, Vec<S>>],
) -> String
where
    S: Borrow<str> + std::fmt::Display,
{
    use prettytable::{Cell, Row, Table};

    let mut table = table_with_title_bar(search_by_in_title);

    // Performance: It will look up for every iteration below.
    // For this reason a hash map is used to make the lookup fast.
    let search_by_in_title: HashMap<&str, usize> = search_by_in_title
        .iter()
        .enumerate()
        .map(|(index, &key)| (key, index))
        .collect();

    for row_to_convert in attrs.iter() {
        // Construct with initial empty cells
        let mut cells = vec![Cell::new(""); search_by_in_title.len()];

        for (cell_name, cell_values) in row_to_convert.iter() {
            // By coupling the field name to value as index for a row, the returned values
            // do not get mixed up in the wrong field names
            if let Some(&index) = search_by_in_title.get(cell_name.borrow()) {
                let cell_v = cell_values.join(" | ");
                *cells.get_mut(index).unwrap() = Cell::new(&cell_v);
            } else {
                warn!(
                    "Ldap search returned values for the not specified name {}.
                    These values are not included in the table.",
                    cell_name
                );
            }
        }

        table.add_row(Row::new(cells));
    }

    return table.to_string();

    fn table_with_title_bar(fiedl_names: &[&str]) -> Table {
        let mut table = Table::new();

        // Title bar
        let title_cells = fiedl_names
            .iter()
            .map(|to_cell| Cell::new(to_cell))
            .collect();

        table.set_titles(Row::new(title_cells));

        table
    }
}
