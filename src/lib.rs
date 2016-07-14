extern crate iron;

use std::collections::HashMap;

use std::sync::{Arc, RwLock};

use std::fmt::Debug;
use std::any::Any;

use iron::prelude::*;
use iron::{typemap, BeforeMiddleware};

impl<T: Clone + Debug + Any> typemap::Key for Util<T> { type Value = Util<T>; }

#[derive(Debug, Clone)]
struct Store<T: Clone + Debug>(Arc<RwLock<HashMap<String, T>>>);

impl<T: Clone + Debug> Store<T> {
  
    pub fn new() -> Self {
        Store(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn get(&self, key: &str) -> Option<T> {
        self.0.read().iter()
            .filter_map(|g| (*g).get(key) )
            .cloned()
            .next()
    }

    fn insert(&self, key: String, value: T) {
        if let Ok(mut lock) = self.0.write() {
            (*lock).insert(key, value);
        }
    }
}

pub struct Builder<T: Clone + Debug> {
    key: Box<Fn(&mut Request) -> String + Send + Sync>,
    store: Store<T>
}

impl<T: Clone + Debug> Builder<T> {

    pub fn new(key: Box<Fn(&mut Request) -> String + Send + Sync>) -> Self {
        Builder { key: key, store: Store::new() }
    }

}

pub struct Util<T: Clone + Debug> {
    key: String,
    store: Store<T>
}

impl<T: Clone + Debug> Util<T> {

    pub fn get(&self) -> Option<T> {
        self.store.get(&self.key)
    }

    pub fn set(&self, value: T) {
        self.store.insert(self.key.clone(), value);
    }

}

impl<T: Clone + Debug + Send + Sync + Any> BeforeMiddleware for Builder<T> {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        let key = (self.key)(req);
        let util = Util { key: key, store: self.store.clone() };
        req.extensions.insert::<Util<T>>(util);
        Ok(())
    }
}

#[cfg(feature="helpers")]
pub mod helpers {
    
    extern crate rand;
    extern crate rustc_serialize;
    extern crate cookie_fe;

    use self::rand::{thread_rng, Rng};
    use self::cookie_fe::{Util as CookieUtil, CookiePair};
    use self::rustc_serialize::hex::ToHex;
    use super::iron::Request;

    fn random() -> String {
        let mut v = [0; 16];
        thread_rng().fill_bytes(&mut v);
        v.to_hex()
    }

    pub fn key_gen(sid: Option<&'static str>) -> Box<Fn(&mut Request) -> String + Send + Sync> {
        let out = move |req: &mut Request| -> String {
            let jar = req.extensions.get_mut::<CookieUtil>()
                .and_then(|x| x.jar() )
                .expect("No cookie jar");
            let sid = sid.unwrap_or("IRONSID");
            if let Some(cookie) = jar.signed().find(sid) {
                cookie.value
            } else {
                let key = random();
                let cookie = CookiePair::new(sid.to_owned(), key.clone());
                jar.signed().add(cookie);
                key
            }
        };
        Box::new(out)        
    }
}
