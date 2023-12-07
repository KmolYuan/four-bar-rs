//! Functions for reading/writing CSV format.
pub use csv::Error;
use csv::{ReaderBuilder, Writer};
use std::io::ErrorKind::InvalidData;

/// Parse CSV from a reader.
pub fn from_reader<R, D>(r: R) -> Result<Vec<D>, Error>
where
    R: std::io::Read,
    D: serde::de::DeserializeOwned,
{
    ReaderBuilder::new()
        .has_headers(false)
        .comment(Some(b'#'))
        .from_reader(r)
        .deserialize()
        .collect::<Result<Vec<_>, _>>()
        .and_then(|data| match data.is_empty() {
            true => Err(std::io::Error::new(InvalidData, "Empty data"))?,
            false => Ok(data),
        })
}

/// Parse CSV from string.
pub fn from_string<D>(s: &str) -> Result<Vec<D>, Error>
where
    D: serde::de::DeserializeOwned,
{
    from_reader(s.as_bytes())
}

/// Dump CSV to a writer.
pub fn to_writer<W, C, S>(w: W, c: C) -> Result<(), csv::Error>
where
    W: std::io::Write,
    C: AsRef<[S]>,
    S: serde::Serialize,
{
    let mut w = Writer::from_writer(w);
    c.as_ref().iter().try_for_each(|c| w.serialize(c))?;
    w.flush()?;
    Ok(())
}

/// Dump CSV to string.
pub fn to_string<C, S>(c: C) -> Result<String, csv::Error>
where
    C: AsRef<[S]>,
    S: serde::Serialize,
{
    let mut w = Vec::new();
    to_writer(&mut w, c)?;
    Ok(String::from_utf8(w).unwrap())
}
