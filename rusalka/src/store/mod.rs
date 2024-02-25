use std::ops::{Deref, DerefMut};
use std::sync::{atomic::AtomicU64, Arc, Mutex, Weak, MutexGuard};


/// Unsubscribes from the store when dropped
pub trait StoreUnsubscribe: Drop {}

pub trait Signal {
    fn subscribe(&self, callback: Box<dyn FnMut()>) -> Box<dyn StoreUnsubscribe>;
}

pub trait ReadableStore: Signal {
    type State;
    fn get(&self) -> &Self::State;
}

pub trait WritableStore: ReadableStore {
    fn set(&mut self, state: Self::State);
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

pub struct Readable<T> {
    state: T,
    listeners: Arc<Mutex<Vec<Listener>>>
}

impl<T> Readable<T> {
    pub fn new(state: T) -> Self {
        Self {
            state,
            listeners: Arc::new(Mutex::new(Vec::new()))
        }
    }
}

static CALL_COUNT: AtomicU64 = AtomicU64::new(0);

impl<T: 'static> Signal for Readable<T> {
    fn subscribe(&self, mut callback: Box<dyn FnMut()>) -> Box<dyn StoreUnsubscribe> {
        let hash = CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut listeners = self.listeners.lock().unwrap();
        callback();
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

impl<T: 'static> ReadableStore for Readable<T> {
    type State = T;
    fn get(&self) -> &Self::State {
        &self.state
    }
}

pub struct Writable<T> {
    state: T,
    listeners: Arc<Mutex<Vec<Listener>>>
}

impl<T> Writable<T> {
    pub fn new(state: T) -> Self {
        Self {
            state,
            listeners: Arc::new(Mutex::new(Vec::new()))
        }
    }
}

impl<T: 'static> Signal for Writable<T> {
    fn subscribe(&self, mut callback: Box<dyn FnMut()>) -> Box<dyn StoreUnsubscribe> {
        let hash = CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut listeners = self.listeners.lock().unwrap();
        callback();
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
    fn get(&self) -> &Self::State {
        &self.state
    }
}

impl<T: 'static> WritableStore for Writable<T> {
    fn set(&mut self, state: Self::State) {
        self.state = state;
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

pub struct DerefGuard<T> {
    listeners: Arc<Mutex<Vec<Listener>>>,
    tainted: bool,
    inner: T
}

pub trait DerefGuardExt<T>: WritableStore + Signal {
    fn guard(&mut self) -> DerefGuard<&mut T>;
}

impl <T: 'static> DerefGuardExt<T> for Writable<T> {
    fn guard(&mut self) -> DerefGuard<&mut T> {
        DerefGuard {
            listeners: self.listeners.clone(),
            tainted: false,
            inner: &mut self.state
        }
    }
}

impl<T> Deref for DerefGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for DerefGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tainted = true;
        &mut self.inner
    }
}

impl<T> Drop for DerefGuard<T> {
    fn drop(&mut self) {
        if self.tainted {
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