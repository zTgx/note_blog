extern crate hyper;

use hyper::{Body, Request, Response, Server};
use hyper::rt::Future;
use hyper::service::service_fn_ok;

#[macro_use] extern crate tera;
#[macro_use] extern crate lazy_static;
extern crate serde_json;

use tera::{Tera, Context, Result};
use serde_json::value::{Value, to_value};

use std::collections::HashMap;

const PHRASE: &str = "Hello, World!";
fn hello_world(_req: Request<Body>) -> Response<Body> {
    Response::new(Body::from(PHRASE))
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = compile_templates!("templates/**/*");
        tera.autoescape_on(vec!["html", ".sql"]);
        tera.register_filter("do_nothing", do_nothing_filter);
        tera
    };
}

pub fn do_nothing_filter(value: Value, _: HashMap<String, Value>) -> Result<Value> {
    let s = try_get_value!("do_nothing_filter", "value", String, value);
    Ok(to_value(&s).unwrap())
}

//render templates
pub fn render(_req: Request<Body>) -> Response<Body> {
    let mut context = Context::new();
    context.add("username", &"Bob");
    context.add("numbers", &vec![1,2,3]);
    context.add("show_all", &false);
    context.add("bio", &"<script>alert('pwnd');</script>");

    // A one off template
    Tera::one_off("hello", &Context::new(), true).unwrap();

    match TEMPLATES.render("users/profile.html", &context) {
        Ok(s) => { 
            println!("{:?}", s);
            
            return Response::new(Body::from(s));
    
        },
        
        Err(e) => {
            println!("Error: {}", e);
            for e in e.iter().skip(1) {
                println!("Reason: {}", e);
            }
        },
    };

    Response::new(Body::from(PHRASE))
}

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    println!("version : {}", version);

    // This is our socket address...
    let addr = ([127, 0, 0, 1], 3000).into();

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let new_svc = || {
        // service_fn_ok converts our function into a `Service`
        service_fn_ok(render)
        //service_fn_ok(hello_world)
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    // Run this server for... forever!
    println!("server running...");
    hyper::rt::run(server);
}
