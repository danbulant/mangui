use std::{collections::HashMap, fmt::Debug, sync::{Arc, Mutex}};
// use crate::nodes::Node;
use super::NodeEvent;

/// A node event handler
pub type EventHandler = dyn FnMut(&NodeEvent) + Send;

pub type InnerEventHandlerDataset = Arc<Mutex<HashMap<usize, Arc<Mutex<Box<EventHandler>>>>>>;

/// An event handler database that allows adding and removing event handlers and running them all.
/// **IMPORTANT**: handlers are locked during event execution, so you can't access handlers from within an event handler.
/// Debug output of EventHandlerDatabase is changed as to prevent deadlocks - handlers are not printed.
/// Although Arc<Mutex> is used, you may be able to delay changing handlers by using thread/some async runtime. I didn't check it though :)
#[derive(Default)]
pub struct EventHandlerDatabase {
    pub handlers: InnerEventHandlerDataset,
    next_token: usize,
}

impl Debug for EventHandlerDatabase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventHandlerDatabase")
            .field("handlers", &"Disabled as it's too easy to get a deadlock :(")
            .field("next_token", &self.next_token)
            .finish()
    }
}

impl EventHandlerDatabase {
    /// Creates a new event handler database with the given handlers.
    /// If you don't need to add or remove handlers, you can use [EventHandlerDatabase::default] instead.
    pub fn new(handlers: Vec<Box<EventHandler>>) -> Self {
        let mut db = Self::default();
        for handler in handlers {
            db.add_handler(handler);
        }
        db
    }

    /// Adds an event handler to the database and returns a token that can be used to remove it.
    pub fn add_handler(&mut self, handler: Box<EventHandler>) -> usize {
        let token = self.next_token;
        self.next_token += 1;
        self.handlers.lock().unwrap().insert(token, Arc::new(Mutex::new(handler)));
        token
    }

    /// Removes an event handler from the database using the token returned by [EventHandlerDatabase::add_handler].
    pub fn remove_handler(&mut self, token: usize) {
        self.handlers.lock().unwrap().remove(&token);
    }
}
