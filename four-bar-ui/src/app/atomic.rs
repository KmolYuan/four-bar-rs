use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{atomic::*, Arc};

macro_rules! impl_serde {
    ($(fn $ty:ty => ($func_ser:ident, $func_de:ident))+) => {$(
        pub(crate) fn $func_ser<S>(atom: &Arc<$ty>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            atom.load(Ordering::Relaxed).serialize(serializer)
        }

        pub(crate) fn $func_de<'a, D>(deserializer: D) -> Result<Arc<$ty>, D::Error>
        where
            D: Deserializer<'a>,
        {
            Ok(Arc::new(<$ty>::new(Deserialize::deserialize(deserializer)?)))
        }
    )+};
}

impl_serde! {
    fn AtomicU64 => (serialize_u64, deserialize_u64)
}
