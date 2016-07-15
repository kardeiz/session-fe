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
use session_fe::{Util as SessionUtil, Builder as SessionBuilder, helpers as session_helpers};

use std::sync::Arc;

use std::collections::BTreeMap;

use rustc_serialize::json::{self, ToJson};
use rustc_serialize::hex::ToHex;

const KEY: &'static [u8] = b"4b8eee793a846531d6d95dd66ae48319";


fn set(req: &mut Request) -> IronResult<Response> {

    let res = Response::with((status::Ok, "Set session"));

    let session_util = iexpect!(req.extensions.get::<SessionUtil<json::Object>>());

    let mut map = json::Object::new();

    map.insert(format!("{}", time::now().rfc3339()), "now".to_json());

    session_util.set(map);

    // iexpect!(req.extensions.get::<SessionUtil<_>>())
    //     .set(map);

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
    let sessioning = SessionBuilder::<json::Object>::new(session_helpers::key_gen(None));

    let mut router = Router::new();
    
    router
        .get("/set", set)
        .get("/get", get);

    let mut chain = Chain::new(router);

    chain.link_before(sessioning);

    let wrapped = CookieBuilder::new(KEY).around(Box::new(chain));

    Iron::new(wrapped).http("0.0.0.0:3000").unwrap();
}
