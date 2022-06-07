use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(PartialEq, Eq, Debug)]
enum HttpMethod {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH
}

type RouteHandler = fn(req: &dyn FlorenceRequest, res: &mut dyn FlorenceResponse);

pub trait FlorenceResponse {
    fn set_status(&mut self, code: u32);
    fn set_body(&self, content: String);
    fn send(&self);
}

pub trait FlorenceRequest {
    fn get_method(&self) -> &HttpMethod;
}

pub trait Router {
    fn get(&mut self, uri: String, handler: RouteHandler);
}

pub trait Server {
    fn handle_connection(&self, stream: &mut TcpStream);
    fn start(self, port: u32) -> Result<(), String>;
}

pub struct Florence {
    routes: Vec<Route>
}

impl Florence {
    pub fn new() -> Self {
        Florence {
            routes: vec![]
        }
    }
}

impl Router for Florence {
    fn get(&mut self, uri: String, handler: RouteHandler) {
        println!("GET {}", uri);
        let route = Route::new(HttpMethod::GET, uri, handler);
        self.routes.push(route);
    }
}

impl Server for Florence {
    fn handle_connection(&self, stream: &mut TcpStream) {
        let uri = "/".to_string();

        // read stream
        let mut buffer = [0; 1024*12]; // 8k (apache max header size) + 4k start line
        let read_result = stream.read(&mut buffer);
        if read_result.is_err() {
        // TODO: return Err(format!("Could not parse request: {}", read_result.err().unwrap().to_string()));
        }
        let http_request = String::from_utf8_lossy(&buffer[..]);

        let parse_result= parse_request(http_request.to_string());
        let request = parse_result.unwrap();
        let mut response = Response::new();

        println!("request: {:?}", request);

        let content = "Hello World\n".to_string();
        let http_response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", content.len(), content);
        stream.write(http_response.as_bytes()).unwrap();
        stream.flush().unwrap();

        for route in self.routes.iter() {
            (route.handler)(&request, &mut response);
        }

    }

    fn start(self, port: u32) -> Result<(), String> {
        return match TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(listener) => {
                println!("Listening on port {}", port);
                for mut stream in listener.incoming() {
                    let mut stream = stream.unwrap();
                    println!("Connection established!");
                    self.handle_connection(&mut stream);
                }
                Ok(())
            }
            Err(err) => {
                Err("Failed to start server".to_string())
            }
        };
    }
}

#[derive(Debug)]
pub struct Request {
    body: String,
    headers: HashMap<String,String>,
    method: HttpMethod,
    uri: String,
}

impl FlorenceRequest for Request {
    fn get_method(&self) -> &HttpMethod {
        return &self.method;
    }
}

impl FlorenceRequest for &Request {
    fn get_method(&self) -> &HttpMethod {
        return self.get_method();
    }
}

impl Request {
    fn new(method: HttpMethod, uri: String, body: String, headers: HashMap<String, String>) -> Request {
        Request {
            body,
            headers,
            method,
            uri,
        }
    }
}

pub struct Response {
    status_code: u32
}

impl FlorenceResponse for Response {
    fn set_status(&mut self, code: u32) {
        self.status_code = code;
    }

    fn set_body(&self, content: String) {
        println!("set body todo")
    }

    fn send(&self) {
        println!("set body send")
    }
}

impl Response {
    fn new() -> Response {
        Response { status_code: 200 }
    }
}


pub struct Route {
    handler: RouteHandler,
    method: HttpMethod,
    uri: String,
}

impl Route {
    fn new(method: HttpMethod, uri: String, handler: RouteHandler) -> Route {
        Route {
            handler,
            method,
            uri
        }
    }
}

struct RouteMatch {
    params: HashMap<String,String>,
    route: Route,
}

impl RouteMatch {
    fn new(route: Route, params: HashMap<String, String>) -> RouteMatch {
        RouteMatch {
            route,
            params,
        }
    }
}

struct StartLine {
    method: String,
    uri: String,
    version: String,
}

fn match_route(request: Request, route: Route) -> Option<RouteMatch> {
    // todo
    // split request.uri and route.uri by /
    // string compare vec entries, watch for * wildcard and :parameter placeholders
    // * -> matches anything
    // /foo/:id -> matches /foo/3 but not /foo or /foo/
    // /foo/*/bar -> matches /foo/anything/bar
    // /foo/*blah -> matches (literally) /foo/*blah, no wildcard
    return Some(RouteMatch::new(route, HashMap::new()));
}

fn parse_start_line(start_line: String) -> Result<StartLine, String> {
    let line_parts: Vec<&str> = start_line.split(' ').collect();
    return Ok(StartLine{
        method: line_parts[0].to_string(),
        uri: line_parts[1].to_string(),
        version: line_parts[2].to_string()
    });
}

fn parse_request(http_request: String) -> Result<Request,String> {
    let mut request_lines: Vec<&str> = http_request.split("\r\n").collect();
    let mut headers: HashMap<String, String> = HashMap::new();
    // parse the first line
    let start_line_result = parse_start_line(request_lines[0].to_string());
    if start_line_result.is_err() {
        return Err(start_line_result.err().unwrap())
    }
    let start_line = start_line_result.unwrap();

    let method_result = string_to_http_method(&start_line.method);
    if method_result.is_err() {
        return Err(format!("Failed to parse HTTP request: {}", method_result.err().unwrap()));
    }

    // parse the headers
    let mut i:usize = 0;
    loop {
        i+=1;
        // check of end of headers
        if request_lines.len() <= i || request_lines[i] == "" {
            break;
        }
        // parse header
        let header_parts: Vec<&str> = request_lines[i].split(": ").collect();
        if header_parts.len() != 2 {
            return Err(format!("Invalid header: {}", request_lines[i].to_string()));
        }
        headers.insert(header_parts[0].to_string(), header_parts[1].to_string());
    }
    // gather remaining lines as body content
    let body_vec: Vec<&str> = request_lines.splice(i..request_lines.len(),[]).collect();
    let body = body_vec.join("\r\n").trim_matches(char::from(0)).to_string();

    Ok(Request{
        body,
        headers,
        method: method_result.unwrap(),
        uri: start_line.uri,
    })
}

fn string_to_http_method(method: &String) -> Result<HttpMethod, String> {
    return match method.to_uppercase().as_str() {
        "GET" => Ok(HttpMethod::GET),
        "HEAD" => Ok(HttpMethod::HEAD),
        "POST" => Ok(HttpMethod::POST),
        "PUT" => Ok(HttpMethod::PUT),
        "DELETE" => Ok(HttpMethod::DELETE),
        "CONNECT" => Ok(HttpMethod::CONNECT),
        "OPTIONS" => Ok(HttpMethod::OPTIONS),
        "TRACE" => Ok(HttpMethod::TRACE),
        "PATCH" => Ok(HttpMethod::PATCH),
        _ => Err(format!("Invalid request method"))
    }
}
