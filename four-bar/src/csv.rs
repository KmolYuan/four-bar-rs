//! Functions for reading/writing CSV format.
pub use csv::Error;
use csv::{ReaderBuilder, Writer};
use serde::{de::DeserializeOwned, Serialize};
use std::borrow::Cow;

/// Parse CSV from string.
pub fn parse_csv<D, R>(r: R) -> Result<Vec<D>, Error>
where
    R: std::io::Read,
    D: DeserializeOwned,
{
    let data = ReaderBuilder::new()
        .has_headers(false)
        .comment(Some(b'#'))
        .from_reader(r)
        .deserialize()
        .collect::<Result<Vec<_>, _>>()?;
    if data.is_empty() {
        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "No data").into())
    } else {
        Ok(data)
    }
}

/// Dump CSV to string.
pub fn dump_csv<'a, C, S>(c: C) -> Result<String, Box<dyn std::error::Error>>
where
    Cow<'a, [S]>: From<C>,
    S: Serialize + Clone + 'a,
{
    let mut w = Writer::from_writer(Vec::new());
    let v = Cow::from(c).into_owned();
    v.into_iter().try_for_each(|c| w.serialize(c))?;
    Ok(String::from_utf8(w.into_inner()?)?)
}
