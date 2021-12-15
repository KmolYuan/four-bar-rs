#[cfg(not(target_arch = "wasm32"))]
use atomic::{Atomic as InnerAtomic, Ordering};
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::sync::RwLock;

#[derive(Clone, Default)]
pub(crate) struct Atomic<T: Copy> {
    #[cfg(not(target_arch = "wasm32"))]
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
