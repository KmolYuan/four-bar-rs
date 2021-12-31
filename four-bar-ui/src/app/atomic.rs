use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Default, Deserialize, Serialize)]
pub(crate) struct Atomic<T: Copy> {
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(serialize_with = "self::serde_atomic::serialize")]
    #[serde(deserialize_with = "self::serde_atomic::deserialize")]
    #[serde(bound(
        serialize = "T: Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))]
    inner: Arc<atomic::Atomic<T>>,
    #[cfg(target_arch = "wasm32")]
    inner: Arc<std::sync::RwLock<T>>,
}

impl<T: Copy> From<T> for Atomic<T> {
    fn from(v: T) -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            inner: Arc::new(atomic::Atomic::new(v)),
            #[cfg(target_arch = "wasm32")]
            inner: Arc::new(std::sync::RwLock::new(v)),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl<T: Copy> Atomic<T> {
    pub(crate) fn load(&self) -> T {
        *self.inner.read().unwrap()
    }

    pub(crate) fn store(&self, v: T) {
        *self.inner.write().unwrap() = v;
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Copy> Atomic<T> {
    pub(crate) fn load(&self) -> T {
        self.inner.load(atomic::Ordering::Relaxed)
    }

    pub(crate) fn store(&self, v: T) {
        self.inner.store(v, atomic::Ordering::Relaxed)
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod serde_atomic {
    use atomic::{Atomic, Ordering};
    use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
    use std::sync::Arc;

    pub(super) fn serialize<S, T>(atom: &Arc<Atomic<T>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Copy + Serialize,
        S: Serializer,
    {
        atom.load(Ordering::Relaxed).serialize(serializer)
    }

    pub(super) fn deserialize<'a, D, T>(deserializer: D) -> Result<Arc<Atomic<T>>, D::Error>
    where
        T: Copy + DeserializeOwned,
        D: Deserializer<'a>,
    {
        let v = Deserialize::deserialize(deserializer)?;
        Ok(Arc::new(Atomic::new(v)))
    }
}
