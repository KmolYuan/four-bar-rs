use super::{FourBar, SFourBar};
use serde::ser::*;

macro_rules! ser_fields {
    ($s: ident, $obj: expr $(, $fields: ident)+ $(,)?) => {
        $($s.serialize_field(stringify!($fields), &$obj.$fields)?;)+
    };
}

/// Flatten is unsupported in RON, so we have to manually implement it.
impl Serialize for FourBar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("FourBar", 9)?;
        ser_fields!(s, self.unnorm, p0x, p0y, a);
        ser_fields!(s, self, l1);
        ser_fields!(s, self.unnorm, l2);
        ser_fields!(s, self, l3, l4, l5, g, stat);
        s.end()
    }
}

/// Flatten is unsupported in RON, so we have to manually implement it.
impl Serialize for SFourBar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("SFourBar", 13)?;
        ser_fields!(s, self.unnorm, ox, oy, oz, r, p0i, p0j, a);
        ser_fields!(s, self, l1, l2, l3, l4, l5, g, stat);
        s.end()
    }
}
