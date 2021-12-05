#[cfg(not(target_arch = "wasm32"))]
use atomic::{Atomic as InnerAtomic, Ordering};
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;

#[derive(Clone, Default)]
pub(crate) struct Atomic<T: Copy> {
    #[cfg(not(target_arch = "wasm32"))]
    inner: Arc<InnerAtomic<T>>,
    #[cfg(target_arch = "wasm32")]
    inner: Arc<Mutex<T>>,
}

impl<T: Copy> Atomic<T> {
    pub(crate) fn new(v: T) -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            inner: Arc::new(InnerAtomic::new(v)),
            #[cfg(target_arch = "wasm32")]
            inner: Arc::new(Mutex::new(v)),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn load(&self) -> T {
        self.inner.load(Ordering::Relaxed)
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn load(&self) -> T {
        self.inner.lock().unwrap().clone()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn store(&self, v: T) {
        self.inner.store(v, Ordering::Relaxed);
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn store(&self, v: T) {
        *self.inner.lock().unwrap() = v;
    }
}
