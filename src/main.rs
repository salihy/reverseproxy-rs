pub mod route;

use reverseproxy_rs::ThreadPool;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;
use lazy_static::lazy_static;


lazy_static! {
    static ref CFG_STR: String = fs::read_to_string("config.json").expect("Should have been able to read the file");
    static ref ROUTES_CFG: route::Routes = serde_json::from_str(&CFG_STR).unwrap();
}

fn main() {

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(12);

    for stream in listener.incoming() {
        let stream = stream.unwrap();


        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn get_routes() -> route::Routes {
    ROUTES_CFG.clone()
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = vec![0; 4096];
    stream.read(&mut buffer).unwrap();


    let req_str = std::str::from_utf8(&buffer).expect("valid utf8");
    let mut downstream_req_str : String = String::new();

    println!("Req:\n{}", &req_str);

    let method_line: &str = req_str.lines().next().unwrap();
    println!("method_line: {}", method_line);
    let method_lines: Vec<&str> = req_str.lines().filter(|&x| x.starts_with("GET")).collect();

    //todo: GET harici metodlar icin patlar burasi
    if method_lines.len() != 1 {
        println!("no method lines found!");
        return;
    }

    let method_line = method_lines[0];

    let first_line_tokens: Vec<&str> = method_line.split(' ').collect();

    let mut get_routes: Vec<route::Route> = get_routes()
                                            .routes
                                            .into_iter()
                                            .filter(|x| x.downstream_method == first_line_tokens[0])
                                            .collect::<Vec<route::Route>>();
    get_routes = get_routes.into_iter().filter(|x| first_line_tokens[1] == &x.upstream_path).collect();
    
    if get_routes.len() != 1 {
        println!("no routes found!");
        return;
    }

    let get_route: &route::Route = &get_routes[0];

    println!("Route: {:?}", &get_route);

    for line in req_str.lines() {
        if line == method_line {
            downstream_req_str = format!("{}\n{} {} {}", downstream_req_str, get_route.downstream_method, get_route.downstream_path, "HTTP/1.1");
        }
        else {
            downstream_req_str = format!("{}\n{}", downstream_req_str, line);
        }
    }

    match TcpStream::connect("127.0.0.1:7979") {
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

}