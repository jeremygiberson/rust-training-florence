#![feature(str_split_as_str)]
#![feature(iter_intersperse)]
extern crate core;

use crate::florence::{Florence, FlorenceRequest, FlorenceResponse, Router, Server};

mod florence;
mod router_tests;
mod mock_tcp_stream;

fn main() {
    let mut f = Florence::new();
    f.get("/".to_string(), |_req: &dyn FlorenceRequest, res: &mut dyn FlorenceResponse|{
        println!("serving /");
        res.set_status(200);
        res.set_body("Hello /!".to_string());
        res.send();
    });

    f.get("/foo".to_string(), |_req: &dyn FlorenceRequest, res: &mut dyn FlorenceResponse|{
        println!("serving /foo");
        res.set_status(200);
        res.set_body("Hello Foo".to_string());
        res.send();
    });

    let result = f.start(3030);
    if result.is_err() {
        println!("Server error: {:?}", result.err())
    }

}
