extern crate hyper;
extern crate url;
use hyper::{Body, Method, Request, Response, Server};
use hyper::rt::Future;

#[macro_use] extern crate tera;
#[macro_use] extern crate lazy_static;
extern crate serde_json;
use tera::{Tera, Context, Result};
use serde_json::value::{Value, to_value};

use std::collections::HashMap;
use url::form_urlencoded;

extern crate futures;
use futures::{future, Stream};

use hyper::service::service_fn;

mod common;
use common::sql::*;

const PHRASE: &str = "Hello, World!";

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
pub fn render(req: Request<Body>) -> Box<dyn Future<Item=Response<Body>, Error=hyper::Error> + Send> {
    match req.method() {
        &Method::POST => {
            return Box::new( req.into_body().concat2().map(|b| {
                        let params = form_urlencoded::parse(b.as_ref()).into_owned().collect::<HashMap<String, String>>();
//                        println!("params : {:#?}", params);

                        let mut name = "".to_string();
                        if let Some(x) = params.get("name") { name = x.to_string(); };
                        let mut message= "message".to_string();
                        if let Some(x) = params.get("message") { message = x.to_string(); };

                        let body = format!("Hello {}, you saying is {}", name, message);
                        SQLConn::insert(name, Some(message));
                        Response::new(body.into())

                        }));

            /*
               let mut response = Response::new(Body::empty());
               let mapping = req.into_body().map(|chunk| {
               chunk
               .iter()
               .map(|byte| byte.to_ascii_uppercase())
               .collect::<Vec<u8>>()
               });

             *response.body_mut() = Body::wrap_stream(mapping);
             return response;
             */

            //echo
            /*
               let mut response = Response::new(Body::empty());
             *response.body_mut() = req.into_body();
             return response;
             */
            //return Response::new(Body::from("helllll"));
        },

            &Method::GET => {
                let mut context = Context::new();
                // A one off template
                Tera::one_off("hello", &Context::new(), true).unwrap();

                match TEMPLATES.render("index.html", &context) {
                    Ok(s) => { 
                        return Box::new(future::ok(Response::new(Body::from(s))));
                    },

                        Err(e) => {
                            for e in e.iter().skip(1) {
                                println!("Reason: {}", e);
                            }
                        },
                };
            },

            _ => {},

    }

    //default rep
    Box::new(future::ok(Response::new(Body::from(PHRASE))))
}

fn init_sql() {
}

fn run() {
    let addr = ([127, 0, 0, 1], 3000).into();

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let new_svc = || { service_fn(render) };

    let server = Server::bind(&addr).serve(new_svc).map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server);
}

fn config() {
    let version = env!("CARGO_PKG_VERSION");
    println!("version : {}", version);
}

fn main() {
    config();

    init_sql();

    run();
}
