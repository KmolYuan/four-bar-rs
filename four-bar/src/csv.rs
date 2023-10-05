//! Functions for reading/writing CSV format.
pub use csv::Error;
use csv::{ReaderBuilder, Writer};
use serde::{de::DeserializeOwned, Serialize};

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

/// Dump CSV to a writer.
pub fn dump_csv<'a, W, C, S>(w: W, c: C) -> Result<(), csv::Error>
where
    W: std::io::Write,
    C: AsRef<[S]>,
    S: Serialize + Clone + 'a,
{
    let mut w = Writer::from_writer(w);
    c.as_ref().iter().try_for_each(|c| w.serialize(c))?;
    w.flush()?;
    Ok(())
}

/// Dump CSV to string.
pub fn csv_string<'a, C, S>(c: C) -> Result<String, csv::Error>
where
    C: AsRef<[S]>,
    S: Serialize + Clone + 'a,
{
    let mut w = Vec::new();
    dump_csv(&mut w, c)?;
    Ok(String::from_utf8(w).unwrap())
}
