pub mod route;

use reverseproxy_rs::ThreadPool;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;
use route_recognizer::{Router, Params};

use lazy_static::lazy_static;

use crate::route::Route;


lazy_static! {
    static ref CFG_STR: String = fs::read_to_string("config.json").expect("Should have been able to read the file");
    static ref ROUTES_CFG: route::Routes = serde_json::from_str(&CFG_STR).unwrap();
}

fn main() {

    let mut router: Router<Route> = Router::new();

    for route in &ROUTES_CFG.routes {
        router.add(&route.upstream_path, route.clone());
    }

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(12);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let r = router.clone();

        pool.execute(|| {
            handle_connection(stream, r);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream, router: Router<Route>) {
    let mut buffer = vec![0; 4096];
    stream.read(&mut buffer).unwrap();


    let req_str = std::str::from_utf8(&buffer).expect("valid utf8");
    let mut downstream_req_str : String = String::new();

    println!("Req:\n{}", &req_str);

    let method_line: &str = req_str.lines().next().unwrap();
    let first_line_tokens: Vec<&str> = method_line.split(' ').collect();

    match router.recognize(first_line_tokens[1]) {
        Ok(m) => {
            for line in req_str.lines() {
                if line == method_line {
                    
                    let mut ds_path: String = m.handler().downstream_path.to_string();

                    for param in m.params().iter() {
                        ds_path = ds_path.replace(&format!(":{}", param.0), param.1);
                        println!("{}", param.0);
                        println!("{}", param.1);
                    }

                    downstream_req_str = format!("{}\n{} {} {}", 
                                            downstream_req_str, 
                                            m.handler().downstream_method, 
                                            ds_path, 
                                            "HTTP/1.1");
                }
                else {
                    downstream_req_str = format!("{}\n{}", downstream_req_str, line);
                }
            }
        
            match TcpStream::connect(m.handler().downstream_uri.to_string()) {
                Ok(mut client_stream) => {
                    println!("Successfully connected to server in port 7979");
                    client_stream.write(downstream_req_str.as_bytes()).unwrap();
        
                    let mut data = [0 as u8; 4096];
                    match client_stream.read(&mut data) {
                        Ok(_) => {
        
                            println!("Resp:\n{}", std::str::from_utf8(&data).expect("valid utf8"));
        
                            stream.write_all(&data).unwrap();
                            stream.flush().unwrap();
                        },
                        Err(err) => {
                            println!("Err: {:?}", err);
                        }
                    }
                },
                Err(_) => {}
            }
        },
        Err(_) => {}
    }


    

}