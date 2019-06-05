extern crate hyper;
extern crate url;

use hyper::{Body, Method, Request, Response, Server, StatusCode};

use hyper::rt::Future;
use hyper::service::service_fn_ok;

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

extern crate postgres;
use postgres::{Connection, TlsMode};


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
    println!("s: {}", s);
    Ok(to_value(&s).unwrap())
}

//render templates
pub fn render(req: Request<Body>) -> Box<dyn Future<Item=Response<Body>, Error=hyper::Error> + Send> { //Response<Body> {
    match req.method() {
            &Method::POST => {
                println!("post method.");
                println!("params : {:#?}", &req);


                //let map = req.into_body().map(|b| { b.iter().map(|b| b.to_ascii_uppercase() ).collect::<Vec<u8>>()  });
                return Box::new( req.into_body().concat2().map(|b| {
                    let params = form_urlencoded::parse(b.as_ref()).into_owned().collect::<HashMap<String, String>>();
                    println!("params : {:#?}", params);

                    let mut name = "xx".to_string();
                    if let Some(x) = params.get("name") { name = x.to_string(); };
                    let mut message= "message".to_string();
                    if let Some(x) = params.get("message") { message = x.to_string(); };

                let body = format!("Hello {}, your number is {}", name, message);
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
                        /*
                           context.add("username", &"Bob");
                           context.add("numbers", &vec![1,2,3]);
                           context.add("show_all", &false);
                           context.add("bio", &"<script>alert('pwnd');</script>");
                         */
                        // A one off template
                        Tera::one_off("hello", &Context::new(), true).unwrap();

                        match TEMPLATES.render("index.html", &context) {
                            Ok(s) => { 
                                println!("{:?}", s);

                                return Box::new(future::ok(Response::new(Body::from(s))));

                            },

                                Err(e) => {
                                    println!("Error: {}", e);
                                    for e in e.iter().skip(1) {
                                        println!("Reason: {}", e);
                                    }
                                },
                        };
                    },

                    _ => {},

            }

            Box::new(future::ok(Response::new(Body::from(PHRASE))))
    }

    fn test_render() {
        const LIPSUM: &'static str =
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut \
            labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco \
            laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in \
            voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat \
            cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum";

        let tera = Tera::new("templates/**/*");
        let mut ctx = Context::new();
        ctx.add("title", &"hello world!");
        ctx.add("content", &LIPSUM);
        ctx.add("todos",
                &vec!["buy milk", "walk the dog", "write about tera"]);
        match TEMPLATES.render("index.html", &ctx) {
            Ok(s) => {
                println!("OK: {:?}", s);        
            },

                Err(e) => {
                    println!("err: {}", e);
                },
        }
    }

    fn init_sql() {

        struct Person {
            id : i32,
            name: String,
            data: Option<String>,
        }
        let conn = Connection::connect("postgres://postgres:123456@127.0.0.1:5432", TlsMode::None).unwrap();
        conn.execute("CREATE TABLE person (
                                id SERIAL PRIMARY KEY,
                                name VARCHAR NOT NULL,
                                data VARCHAR)", &[]).unwrap();

    //}

    //fn insert() {
        let me = Person {
            id: 0,
            name: "Steven".to_string(),
            data: None,
        };

        conn.execute("INSERT INTO person (name, data) VALUES ($1, $2)",  &[&me.name, &me.data]).unwrap();
    //}

    //fn query() {
        for row in &conn.query("SELECT id, name, data FROM person", &[]).unwrap() {
            let person = Person {
                        id: row.get(0),
                        name: row.get(1),
                        data: row.get(2),
            };

            println!("Found person {}: {}", person.id, person.name);
        }
    }
    fn main() {

        init_sql();

        let version = env!("CARGO_PKG_VERSION");
        println!("version : {}", version);

        // This is our socket address...
        let addr = ([127, 0, 0, 1], 3000).into();

        // A `Service` is needed for every connection, so this
        // creates one from our `hello_world` function.
        let new_svc = || {
            // service_fn_ok converts our function into a `Service`
            service_fn(render)
                //service_fn_ok(hello_world)
        };

        let server = Server::bind(&addr)
            .serve(new_svc)
            .map_err(|e| eprintln!("server error: {}", e));

        // Run this server for... forever!
        println!("server running...");
        hyper::rt::run(server);
    }
