use csv::{Error as CsvError, ReaderBuilder, Writer};
use serde::{de::DeserializeOwned, Serialize};
use std::{error::Error, io::Cursor};

/// Parse CSV from string.
pub(crate) fn parse_csv<D>(s: &str) -> Result<Vec<D>, CsvError>
where
    D: DeserializeOwned,
{
    ReaderBuilder::new()
        .has_headers(false)
        .comment(Some(b'#'))
        .from_reader(Cursor::new(s))
        .deserialize()
        .collect()
}

/// Dump CSV to string.
pub(crate) fn dump_csv<'a, C, S>(c: C) -> Result<String, Box<dyn Error>>
where
    C: Into<std::borrow::Cow<'a, [S]>>,
    S: Serialize + Clone + 'a,
{
    let mut w = Writer::from_writer(Vec::new());
    let v = c.into().into_owned();
    v.into_iter().try_for_each(|c| w.serialize(c))?;
    Ok(String::from_utf8(w.into_inner()?)?)
}
