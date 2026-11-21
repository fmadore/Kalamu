//! Rows copied from one database to another, table by table, by column name: the store into a
//! project file when saving, a project file into the store when opening. Each row passes
//! through a function that may change it (a job kept to some of its sources, identifiers given
//! new ones).

use rusqlite::types::Value;
use rusqlite::{Connection, ToSql};

/// Copies the rows of `table` that `filter` keeps (`WHERE …`, or empty) from `from` to `to`,
/// each through `each`, which is given the columns' names. Only the named arguments the filter
/// mentions are bound. With `or_ignore`, a row whose key `to` already holds is left as it is
/// there. Returns how many rows were read.
pub(super) fn copy(
    from: &Connection,
    to: &Connection,
    table: &str,
    filter: &str,
    args: &[(&str, &dyn ToSql)],
    or_ignore: bool,
    mut each: impl FnMut(&[String], &mut Vec<Value>),
) -> rusqlite::Result<u64> {
    let sql = format!("SELECT * FROM \"{table}\" {filter}");
    let mut select = from.prepare(&sql)?;
    let columns: Vec<String> = select.column_names().map(str::to_string).collect();
    let bound: Vec<(&str, &dyn ToSql)> = args
        .iter()
        .filter(|(name, _)| filter.contains(name))
        .copied()
        .collect();
    let names = columns
        .iter()
        .map(|c| format!("\"{c}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let marks = (1..=columns.len())
        .map(|i| format!("?{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let verb = if or_ignore {
        "INSERT OR IGNORE"
    } else {
        "INSERT"
    };
    let mut insert = to.prepare(&format!(
        "{verb} INTO \"{table}\" ({names}) VALUES ({marks})"
    ))?;
    let mut rows = select.query(bound.as_slice())?;
    let mut read = 0;
    while let Some(row) = rows.next()? {
        let mut values = (0..columns.len())
            .map(|i| row.get::<_, Value>(i))
            .collect::<rusqlite::Result<Vec<_>>>()?;
        each(&columns, &mut values);
        insert.execute(rusqlite::params_from_iter(values.iter()))?;
        read += 1;
    }
    Ok(read)
}

/// The index of a column by its name.
pub(super) fn column(columns: &[String], name: &str) -> Option<usize> {
    columns.iter().position(|c| c == name)
}
