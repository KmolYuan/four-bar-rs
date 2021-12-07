use csv::{Error as CsvError, Reader, Writer};
use serde::{de::DeserializeOwned, Serialize};
use std::{error::Error, io::Cursor};

/// Parse CSV from string.
pub fn parse_csv<D>(s: &str) -> Result<Vec<D>, CsvError>
where
    D: DeserializeOwned,
{
    Reader::from_reader(Cursor::new(s)).deserialize().collect()
}

/// Dump CSV to string.
pub fn dump_csv<S>(arr: &[S]) -> Result<String, Box<dyn Error>>
where
    S: Serialize + Clone,
{
    let mut w = Writer::from_writer(Vec::new());
    for s in arr {
        w.serialize(s.clone())?;
    }
    Ok(String::from_utf8(w.into_inner()?)?)
}
