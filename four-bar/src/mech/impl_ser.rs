// Flatten is unsupported in RON, so we have to manually implement it.
use super::{FourBar, MFourBar, SFourBar};
use serde::{de::*, ser::*};

macro_rules! impl_ser {
    ($ty:ident, $($field:ident $(.$unnorm:ident)?),+) => {
        impl Serialize for $ty {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut s = serializer.serialize_struct(stringify!($ty), 10)?;
                $(s.serialize_field(stringify!($field), &self.$($unnorm.)?$field)?;)+
                s.end()
            }
        }

        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> Result<$ty, D::Error>
            where
                D: Deserializer<'de>,
            {
                const FIELDS: &[&str] = &[$(stringify!($field)),+];
                #[allow(non_camel_case_types)]
                enum Field {
                    $($field),+
                }
                impl<'de> Deserialize<'de> for Field {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where
                        D: Deserializer<'de>,
                    {
                        struct FieldVisitor;
                        impl<'de> Visitor<'de> for FieldVisitor {
                            type Value = Field;
                            fn expecting(&self, w: &mut std::fmt::Formatter) -> std::fmt::Result {
                                write!(w, "fields: {FIELDS:?}")
                            }
                            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                            where
                                E: serde::de::Error,
                            {
                                match v {
                                    $(stringify!($field) => Ok(Field::$field),)+
                                    _ => Err(serde::de::Error::unknown_field(v, FIELDS)),
                                }
                            }
                        }
                        deserializer.deserialize_identifier(FieldVisitor)
                    }
                }
                struct StructVisitor;
                impl<'de> Visitor<'de> for StructVisitor {
                    type Value = $ty;
                    fn expecting(&self, w: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(w, concat!["struct ", stringify!($ty)])
                    }
                    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                    where
                        A: MapAccess<'de>,
                    {
                        // Missing field and duplicated field checkers
                        $(let $field = std::cell::OnceCell::new();)+
                        while let Some(k) = map.next_key()? {
                            match k {
                                $(Field::$field => $field
                                    .set(map.next_value()?)
                                    .map_err(|_| serde::de::Error::duplicate_field(stringify!($field)))?,)+
                            }
                        }
                        let mut fb = std::mem::MaybeUninit::<$ty>::uninit();
                        let fb_ptr = fb.as_mut_ptr();
                        // SAFETY: We only write them and never read them.
                        $(unsafe {
                            (*fb_ptr).$($unnorm.)?$field = $field.into_inner()
                                .ok_or_else(|| serde::de::Error::missing_field(stringify!($field)))?;
                        })+
                        // SAFETY: We have initialized all fields.
                        Ok(unsafe { fb.assume_init() })
                    }
                }
                deserializer.deserialize_struct(stringify!($ty), FIELDS, StructVisitor)
            }
        }
    };
}

impl_ser!(FourBar, p1x.unnorm, p1y.unnorm, a.unnorm, l1, l2.unnorm, l3, l4, l5, g, stat);
impl_ser!(MFourBar, p1x.unnorm, p1y.unnorm, a.unnorm, l1, l2.unnorm, l3, l4, l5, g, e, stat);
impl_ser!(
    SFourBar, ox.unnorm, oy.unnorm, oz.unnorm, r.unnorm, p1i.unnorm, p1j.unnorm, a.unnorm, l1, l2,
    l3, l4, l5, g, stat
);
