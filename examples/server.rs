#[macro_use]
extern crate iron;

extern crate cookie_fe;
extern crate session_fe;
extern crate router;
// extern crate persistent;
extern crate rustc_serialize;
extern crate time;
extern crate rand;

use rand::{thread_rng, Rng};

use iron::prelude::*;
use iron::{status, AroundMiddleware};

use router::Router;

use cookie_fe::{Util as CookieUtil, Builder as CookieBuilder, CookiePair};
use session_fe::{Util as SessionUtil, Builder as SessionBuilder};

use std::collections::{BTreeMap, HashMap};

use rustc_serialize::json::{self, ToJson};
use rustc_serialize::hex::{self, ToHex};

const KEY: &'static [u8] = b"4b8eee793a846531d6d95dd66ae48319";

pub struct Helper;

impl Helper {

    pub fn random() -> String {
        let mut v = [0; 16];
        rand::thread_rng().fill_bytes(&mut v);
        v.to_hex()
    }

    fn key(sid: Option<&'static str>) -> Box<Fn(&mut Request) -> String + Send + Sync> {
        let out = move |req: &mut Request| -> String {
            let jar = req.extensions.get_mut::<CookieUtil>()
                .and_then(|x| x.jar() )
                .expect("No cookie jar");
            let sid = sid.unwrap_or("IRONSID");
            if let Some(cookie) = jar.signed().find(sid) {
                cookie.value
            } else {
                let key = Self::random();
                let cookie = CookiePair::new(sid.to_owned(), key.clone());
                jar.signed().add(cookie);
                key
            }
        };
        Box::new(out)        
    }
}


fn set(req: &mut Request) -> IronResult<Response> {

    let mut res = Response::with((status::Ok, "Set session"));

    let mut map = BTreeMap::new();

    map.insert(format!("{}", time::now().rfc3339()), "now".to_json());

    iexpect!(req.extensions.get::<SessionUtil<_>>())
        .insert(map);

    Ok(res)
}

fn get(req: &mut Request) -> IronResult<Response> {

    let mut res = Response::new();

    let session = iexpect!(req.extensions.get::<SessionUtil<json::Object>>()).get();

    res
        .set_mut(status::Ok)
        .set_mut(format!("{:?}", session));

    Ok(res)
}



fn main() {
    let sessioning = SessionBuilder::<json::Object>::new(Helper::key(None));

    let mut router = Router::new();
    router.get("/set", set);
    router.get("/get", get);
    let mut chain = Chain::new(router);

    chain.link_before(sessioning);

    let wrapped = CookieBuilder(KEY).around(Box::new(chain));

    Iron::new(wrapped).http("0.0.0.0:3000").unwrap();
}
