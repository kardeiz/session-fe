extern crate rustc_serialize;
extern crate iron;

use std::collections::HashMap;

use std::sync::{Arc, RwLock};

use iron::prelude::*;
use iron::{typemap, BeforeMiddleware};

use rustc_serialize::json;

impl typemap::Key for Util { type Value = Self; }

#[derive(Debug, Clone)]
struct Store(Arc<RwLock<HashMap<String, json::Object>>>);

impl Store {
  
    pub fn new() -> Self {
        Store(Arc::new(RwLock::new( HashMap::new())))
    }

    pub fn get(&self, key: &str) -> Option<json::Object> {
        self.0.read().iter()
            .filter_map(|g| (*g).get(key) )
            .map(|v| v.clone() )
            .next()
    }

    fn insert(&self, key: String, value: json::Object) {
        if let Ok(mut lock) = self.0.write() {
            (*lock).insert(key, value);
        }
    }
}

pub struct Builder {
    key: Box<Fn(&mut Request) -> String + Send + Sync>,
    store: Store
}

impl Builder {

    pub fn new(key: Box<Fn(&mut Request) -> String + Send + Sync>) -> Self {
        Builder { key: key, store: Store::new() }
    }

}

pub struct Util {
    key: String,
    store: Store
}

impl Util {

    pub fn get(&self) -> Option<json::Object> {
        self.store.get(&self.key)
    }

    pub fn set(&self, value: json::Object) {
        self.store.insert(self.key.clone(), value);
    }

}

impl BeforeMiddleware for Builder {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        let key = (self.key)(req);
        let util = Util { key: key, store: self.store.clone() };
        req.extensions.insert::<Util>(util);
        Ok(())
    }
}
