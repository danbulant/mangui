use std::sync::{atomic::AtomicU64, Arc, Mutex, Weak};


/// Unsubscribes from the store when dropped
pub trait StoreUnsubscribe: Drop {}

pub trait ReadableStore {
    type State;
    fn get(&self) -> &Self::State;
    fn subscribe(&self, callback: Box<dyn FnMut(&Self::State) -> ()>) -> Box<dyn StoreUnsubscribe>;
}

pub trait WritableStore: ReadableStore {
    fn set(&mut self, state: Self::State);
}

struct Listener<T> {
    hash: u64,
    callback: Box<dyn FnMut(&T) -> ()>
}

struct ReadableUnsubscribe<T> {
    listeners: Weak<Mutex<Vec<Listener<T>>>>,
    hash: u64
}

impl<T: Sized> Drop for ReadableUnsubscribe<T> {
    fn drop(&mut self) {
        if let Some(listeners) = self.listeners.upgrade() {
            let mut listeners = listeners.lock().unwrap();
            listeners.retain(|listener| listener.hash != self.hash);
        }
    }
}

impl<T> StoreUnsubscribe for ReadableUnsubscribe<T> {}

pub struct Readable<T> {
    state: T,
    listeners: Arc<Mutex<Vec<Listener<T>>>>
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

impl<T: 'static> ReadableStore for Readable<T> {
    type State = T;
    fn get(&self) -> &Self::State {
        &self.state
    }
    fn subscribe(&self, callback: Box<dyn FnMut(&Self::State) -> ()>) -> Box<dyn StoreUnsubscribe> {
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

pub struct Writable<T> {
    state: T,
    listeners: Arc<Mutex<Vec<Listener<T>>>>
}

impl<T> Writable<T> {
    pub fn new(state: T) -> Self {
        Self {
            state,
            listeners: Arc::new(Mutex::new(Vec::new()))
        }
    }
}

impl<T: 'static> ReadableStore for Writable<T> {
    type State = T;
    fn get(&self) -> &Self::State {
        &self.state
    }
    fn subscribe(&self, callback: Box<dyn FnMut(&Self::State) -> ()>) -> Box<dyn StoreUnsubscribe> {
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

impl<T: 'static> WritableStore for Writable<T> {
    fn set(&mut self, state: Self::State) {
        self.state = state;
        let mut listeners = self.listeners.lock().unwrap();
        for listener in listeners.iter_mut() {
            (listener.callback)(&self.state);
        }
    }
}