#[cfg(not(target_arch = "wasm32"))]
use atomic::{Atomic as InnerAtomic, Ordering};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::sync::RwLock;

#[derive(Clone, Default, Deserialize, Serialize)]
pub(crate) struct Atomic<T: Copy> {
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(serialize_with = "self::serde_atomic::serialize")]
    #[serde(deserialize_with = "self::serde_atomic::deserialize")]
    #[serde(bound(
        serialize = "T: Serialize",
        deserialize = "T: serde::de::DeserializeOwned"
    ))]
    inner: Arc<InnerAtomic<T>>,
    #[cfg(target_arch = "wasm32")]
    inner: Arc<RwLock<T>>,
}

impl<T: Copy> From<T> for Atomic<T> {
    fn from(v: T) -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            inner: Arc::new(InnerAtomic::new(v)),
            #[cfg(target_arch = "wasm32")]
            inner: Arc::new(RwLock::new(v)),
        }
    }
}

impl<T: Copy> Atomic<T> {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn load(&self) -> T {
        self.inner.load(Ordering::Relaxed)
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn load(&self) -> T {
        *self.inner.read().unwrap()
    }

    pub(crate) fn store(&self, v: T) {
        #[cfg(not(target_arch = "wasm32"))]
        let _ = self.inner.store(v, Ordering::Relaxed);
        #[cfg(target_arch = "wasm32")]
        let _ = *self.inner.write().unwrap() = v;
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
