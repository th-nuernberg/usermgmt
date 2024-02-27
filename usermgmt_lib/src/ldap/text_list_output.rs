use super::ldap_search_result::LdapSearchResult;

/// Returns rows. Every row consists comma separated cells. Every cell is key value pair with "="
/// sign in the middle.
///
/// Example for cell: name=example
///
/// If there is no value for the field, then there is nothing from right of the "=" sign
pub fn ldap_simple_output(search_results: &LdapSearchResult) -> String {
    let attrs = search_results.fields();
    let search_by_in_title = search_results.headers();
    attrs
        .iter()
        .map(|next_row| {
            next_row
                .iter()
                .enumerate()
                .map(|(index, next_field)| {
                    let title = *search_by_in_title
                        .get(index)
                        .expect("Index comes from iterator");
                    format!("{}={}", title, next_field.join("|"))
                })
                .collect::<Vec<String>>()
                .join(",")
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// Returns a pretty ASCII table from the result of an LDAP search query.
/// First row is the title row which shows all used field name.
/// Subsequent rows contain values to each field name for an LDAP entity.
///
/// If there is no value for the field, then the cell of in the column is white space only
pub fn ldap_search_to_pretty_table(search_result: &LdapSearchResult) -> String {
    use prettytable::{Cell, Row, Table};

    let headers = search_result.headers();
    let mut table = table_with_title_bar(headers.as_slice());

    for row_to_convert in search_result.fields().iter() {
        // Construct with initial empty cells
        let mut cells = vec![Cell::new(""); headers.len()];

        for (field_index, cell_value) in row_to_convert.iter().enumerate() {
            let to_mutate = cells
                .get_mut(field_index)
                .expect("Index comes from iterator");
            *to_mutate = Cell::new(&cell_value.join(" | "));
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
