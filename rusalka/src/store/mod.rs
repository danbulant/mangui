use std::ops::{Deref, DerefMut};
use std::ptr::drop_in_place;
use std::sync::{atomic::AtomicU64, Arc, Mutex, Weak, MutexGuard};


/// Unsubscribes from the store when dropped
pub trait StoreUnsubscribe: Drop {}

pub trait Signal {
    fn subscribe(&self, callback: Box<dyn FnMut()>) -> Box<dyn StoreUnsubscribe>;
}

pub trait ReadableStore: Signal {
    type State;
    fn get(&self) -> MutexGuard<Self::State>;
}

pub trait WritableStore: ReadableStore {
    fn set(&self, state: Self::State);
}

struct Listener {
    hash: u64,
    callback: Box<dyn FnMut()>
}
struct ReadableUnsubscribe {
    listeners: Weak<Mutex<Vec<Listener>>>,
    hash: u64
}

impl Drop for ReadableUnsubscribe {
    fn drop(&mut self) {
        if let Some(listeners) = self.listeners.upgrade() {
            let mut listeners = listeners.lock().unwrap();
            listeners.retain(|listener| listener.hash != self.hash);
        }
    }
}

impl StoreUnsubscribe for ReadableUnsubscribe {}

// this isn't actually usable as of now, so it's commented out until I think of a better way to do it
// pub struct Readable<T> {
//     state: T,
//     listeners: Arc<Mutex<Vec<Listener>>>
// }
//
// impl<T> Readable<T> {
//     pub fn new(state: T) -> Self {
//         Self {
//             state,
//             listeners: Arc::new(Mutex::new(Vec::new()))
//         }
//     }
// }

static CALL_COUNT: AtomicU64 = AtomicU64::new(0);

// impl<T: 'static> Signal for Readable<T> {
//     fn subscribe(&self, mut callback: Box<dyn FnMut()>) -> Box<dyn StoreUnsubscribe> {
//         let hash = CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
//         let mut listeners = self.listeners.lock().unwrap();
//         listeners.push(Listener {
//             callback,
//             hash
//         });
//         Box::new(ReadableUnsubscribe {
//             listeners: Arc::downgrade(&self.listeners),
//             hash
//         })
//     }
// }
//
// impl<T: 'static> ReadableStore for Readable<T> {
//     type State = T;
//     fn get(&self) -> &Self::State {
//         &self.state
//     }
// }

pub struct Writable<T> {
    state: Mutex<T>,
    listeners: Arc<Mutex<Vec<Listener>>>
}

impl<T> Writable<T> {
    pub fn new(state: T) -> Self {
        Self {
            state: Mutex::new(state),
            listeners: Arc::new(Mutex::new(Vec::new()))
        }
    }
}

impl<T: 'static> Signal for Writable<T> {
    fn subscribe(&self, mut callback: Box<dyn FnMut()>) -> Box<dyn StoreUnsubscribe> {
        let hash = CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut listeners = self.listeners.lock().unwrap();
        listeners.push(Listener {
            callback,
            hash
        });
        Box::new(ReadableUnsubscribe {
            listeners: Arc::downgrade(&self.listeners),
            hash
        })
    }
}

impl<T: 'static> ReadableStore for Writable<T> {
    type State = T;
    fn get(&self) -> MutexGuard<Self::State> {
        self.state.lock().unwrap()
    }
}

impl<T: 'static> WritableStore for Writable<T> {
    fn set(&self, state: Self::State) {
        *self.state.lock().unwrap().deref_mut() = state;
        let mut listeners = self.listeners.lock().unwrap();
        for listener in listeners.iter_mut() {
            (listener.callback)();
        }
    }
}

impl<T: 'static> Default for Writable<T> where T: Default {
    fn default() -> Self {
        Self::new(Default::default())
    }
}

pub struct DerefGuard<'a, T> {
    listeners: Arc<Mutex<Vec<Listener>>>,
    tainted: bool,
    // this is an option because we need to drop the guard before we trigger the listeners
    // this is only none during the drop function - see https://doc.rust-lang.org/stable/nomicon/destructors.html
    inner: Option<MutexGuard<'a, T>>
}

pub trait DerefGuardExt<T>: WritableStore + Signal {
    fn guard(&self) -> DerefGuard<T>;
}

impl <T: 'static> DerefGuardExt<T> for Writable<T> {
    fn guard(&self) -> DerefGuard<T> {
        DerefGuard {
            listeners: self.listeners.clone(),
            tainted: false,
            inner: Some(self.state.lock().unwrap())
        }
    }
}

impl<'a, T> Deref for DerefGuard<'a, T> {
    type Target = MutexGuard<'a, T>;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl<'a, T> DerefMut for DerefGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tainted = true;
        self.inner.as_mut().unwrap()
    }
}

impl<'a, T> Drop for DerefGuard<'a, T> {
    fn drop(&mut self) {
        if self.tainted {
            let inner = self.inner.take().unwrap();
            drop(inner);

            let mut listeners = self.listeners.lock().unwrap();
            for listener in listeners.iter_mut() {
                (listener.callback)();
            }
        }
    }
}

struct VecUnsub {
    unsubscribes: Vec<Box<dyn StoreUnsubscribe>>
}
impl Drop for VecUnsub {
    fn drop(&mut self) {}
}
impl StoreUnsubscribe for VecUnsub {}


// odd that I have to implement this but whatever makes the compiler happy
impl<T: Signal> Signal for MutexGuard<'_, T> {
    fn subscribe(&self, callback: Box<dyn FnMut()>) -> Box<dyn StoreUnsubscribe> {
        self.deref().subscribe(callback)
    }
}
impl<T: Signal> Signal for Arc<T> {
    fn subscribe(&self, callback: Box<dyn FnMut()>) -> Box<dyn StoreUnsubscribe> {
        self.deref().subscribe(callback)
    }
}

impl<T: Signal> Signal for Vec<T> {
    /// Subscribes to all signals in the vector
    fn subscribe(&self, callback: Box<dyn FnMut()>) -> Box<dyn StoreUnsubscribe> {
        let mut unsubscribes = Vec::with_capacity(self.len());
        let callback = Arc::new(Mutex::new(callback));
        for signal in self.iter() {
            let callback = callback.clone();
            unsubscribes.push(signal.subscribe(Box::new(move || callback.lock().unwrap()())));
        }
        Box::new(VecUnsub { unsubscribes })
    }
}

impl<T: Signal> Signal for [T] {
    /// Subscribes to all signals in the array
    fn subscribe(&self, callback: Box<dyn FnMut()>) -> Box<dyn StoreUnsubscribe> {
        let mut unsubscribes = Vec::with_capacity(self.len());
        let callback = Arc::new(Mutex::new(callback));
        for signal in self.iter() {
            let callback = callback.clone();
            unsubscribes.push(signal.subscribe(Box::new(move || callback.lock().unwrap()())));
        }
        Box::new(VecUnsub { unsubscribes })
    }
}