use std::io::Write;

use serde::ser::SerializeSeq;

macro_rules! impl_disp_methods {
    ($(($method:ident, $ty:ty)),+ $(,)?) => {$(
        fn $method(self, v: $ty) -> Result<Self::Ok, Self::Error> {
            write!(self.writer, "{v}").map_err(Error)
        }
    )+};
}

#[derive(Debug)]
pub(crate) struct Error(std::io::Error);
impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl serde::ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self(std::io::Error::other(msg.to_string()))
    }
}

// A simple TOML-like serializer for the CLI logger
pub(crate) struct Logger<'a, W: Write> {
    writer: &'a mut W,
}

impl<'a, W: Write> Logger<'a, W> {
    pub(crate) fn new(writer: &'a mut W) -> Self {
        Self { writer }
    }

    pub(crate) fn top_title(&mut self, title: &str) -> Result<(), std::io::Error> {
        writeln!(self.writer, "[{title}]")
    }

    pub(crate) fn title(&mut self, title: &'static str) -> Result<(), std::io::Error> {
        writeln!(self.writer, "\n[{title}]")
    }

    pub(crate) fn log<S: serde::Serialize>(&mut self, s: S) -> Result<(), std::io::Error> {
        s.serialize(self).map_err(|e| e.0)
    }

    pub(crate) fn flush(&mut self) -> Result<(), std::io::Error> {
        self.writer.flush()
    }
}

impl<'a, 'b, W: Write> serde::Serializer for &'a mut Logger<'b, W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Seq<'a, 'b, W>;
    type SerializeTuple = Seq<'a, 'b, W>;
    type SerializeTupleStruct = Seq<'a, 'b, W>;
    type SerializeTupleVariant = Seq<'a, 'b, W>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    impl_disp_methods!(
        (serialize_bool, bool),
        (serialize_i8, i8),
        (serialize_i16, i16),
        (serialize_i32, i32),
        (serialize_i64, i64),
        (serialize_u8, u8),
        (serialize_u16, u16),
        (serialize_u32, u32),
        (serialize_u64, u64),
        (serialize_char, char),
        (serialize_str, &str),
        (serialize_unit_struct, &'static str),
    );

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{v:.04}").map_err(Error)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{v:.04}").map_err(Error)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for b in v {
            seq.serialize_element(b)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        write!(self.writer, "{variant}").map_err(Error)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        write!(self.writer, "{name}(").map_err(Error)?;
        value.serialize(&mut *self)?;
        write!(self.writer, ")").map_err(Error)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        write!(self.writer, "{variant}(").map_err(Error)?;
        value.serialize(&mut *self)?;
        write!(self.writer, ")").map_err(Error)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        write!(self.writer, "(").map_err(Error)?;
        Ok(Seq { logger: self, len })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        write!(self.writer, "(").map_err(Error)?;
        Ok(Seq { logger: self, len: Some(len) })
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        write!(self.writer, "{name}(").map_err(Error)?;
        Ok(Seq { logger: self, len: Some(len) })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        write!(self.writer, "{variant}(").map_err(Error)?;
        Ok(Seq { logger: self, len: Some(len) })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(self)
    }
}

pub(crate) struct Seq<'a, 'b, W: Write> {
    logger: &'a mut Logger<'b, W>,
    len: Option<usize>,
}

macro_rules! impl_ser_seq {
    ($(($ty:ident, $method:ident)),+ $(,)?) => {$(
        impl<W: Write> serde::ser::$ty for Seq<'_, '_, W> {
            type Ok = ();
            type Error = Error;

            fn $method<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
            where
                T: ?Sized + serde::Serialize,
            {
                value.serialize(&mut *self.logger)?;
                match &mut self.len {
                    Some(len) if *len == 1 => return Ok(()),
                    Some(len) => *len -= 1,
                    None => (),
                }
                write!(self.logger.writer, ", ").map_err(Error)
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                write!(self.logger.writer, ")").map_err(Error)
            }
        }
    )+};
}

impl_ser_seq!(
    (SerializeSeq, serialize_element),
    (SerializeTuple, serialize_element),
    (SerializeTupleStruct, serialize_field),
    (SerializeTupleVariant, serialize_field),
);

impl<W: Write> serde::ser::SerializeMap for &mut Logger<'_, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        key.serialize(&mut **self)?;
        write!(self.writer, "=").map_err(Error)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)?;
        writeln!(self.writer).map_err(Error)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

macro_rules! impl_ser_map {
    ($($ty:ident),+ $(,)?) => {$(
        impl<W: Write> serde::ser::$ty for &mut Logger<'_, W> {
            type Ok = ();
            type Error = Error;

            fn serialize_field<T>(
                &mut self,
                key: &'static str,
                value: &T,
            ) -> Result<Self::Ok, Self::Error>
            where
                T: ?Sized + serde::Serialize,
            {
                write!(self.writer, "{key}=").map_err(Error)?;
                value.serialize(&mut **self)?;
                writeln!(self.writer).map_err(Error)
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }
        }
    )+};
}

impl_ser_map!(SerializeStruct, SerializeStructVariant);
