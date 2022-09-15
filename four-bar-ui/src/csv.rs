use csv::{Error as CsvError, ReaderBuilder, Writer};
use serde::{de::DeserializeOwned, Serialize};
use std::{error::Error, io::Cursor};

/// Parse CSV from string.
pub fn parse_csv<D>(s: &str) -> Result<Vec<D>, CsvError>
where
    D: DeserializeOwned,
{
    ReaderBuilder::new()
        .has_headers(false)
        .from_reader(Cursor::new(s))
        .deserialize()
        .collect()
}

/// Dump CSV to string.
pub fn dump_csv<'a, C, S>(c: C) -> Result<String, Box<dyn Error>>
where
    C: Into<std::borrow::Cow<'a, [S]>>,
    S: Serialize + Clone + 'a,
{
    let mut w = Writer::from_writer(Vec::new());
    c.into().iter().try_for_each(|s| w.serialize(s.clone()))?;
    Ok(String::from_utf8(w.into_inner()?)?)
}
