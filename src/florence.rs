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

// use std::net::{TcpListener, TcpStream};
// use std::format;
// use std::io::{Read, Write};
// use std::collections::HashMap;
// use std::fmt::{Debug, Formatter};
//
// pub trait FlorenceResponse {
//     fn set_status(self, code: u32);
//     fn set_header(self, name: String, value: String);
//     fn set_body(self, content: String);
//     fn send(self);
// }
//
// pub trait FlorenceRequest {
//     fn get_method(self) -> HttpMethod;
//     fn get_uri(self) -> String;
//     fn get_body(self) -> String;
// }
//
// pub trait Router {
//     fn route(self, method: HttpMethod, uri: String, handler: fn(req: &dyn FlorenceRequest, res: &dyn FlorenceResponse));
//     fn get(self, uri: String, handler: fn(req: &dyn FlorenceRequest, res: &dyn FlorenceResponse));
//     fn post(self, uri: String, handler: fn(req: &dyn FlorenceRequest, res: &dyn FlorenceResponse));
//     fn put(self, uri: String, handler: fn(req: &dyn FlorenceRequest, res: &dyn FlorenceResponse));
//     fn delete(self, uri: String, handler: fn(req: &dyn FlorenceRequest, res: &dyn FlorenceResponse));
//     fn patch(self, uri: String, handler: fn(req: &dyn FlorenceRequest, res: &dyn FlorenceResponse));
//     fn match_route(self, req: &dyn FlorenceRequest) -> Result<Route, String>;
// }
//
// pub trait Server {
//     fn start(self, port:u32) -> Result<(),String>;
// }
//
// struct Response {
//     stream: TcpStream,
//     status: u32,
//     content: String,
//     headers: HashMap<String,String>
// }
//
// #[derive(PartialEq, Eq)]
// enum HttpMethod {
//     GET,
//     HEAD,
//     POST,
//     PUT,
//     DELETE,
//     CONNECT,
//     OPTIONS,
//     TRACE,
//     PATCH
// }
//
// struct Request {
//     method: HttpMethod,
//     uri: String,
// //    stream: TcpStream,
//     body: String,
//     headers: HashMap<String,String>
// }
//
// impl Debug for Request {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Request [uri: {:?}, headers: {:?}, body: {:?}]", self.uri, self.headers, self.body)
//     }
// }
//
// impl FlorenceResponse for Response {
//     fn set_status(mut self, code: u32) {
//         self.status = code;
//     }
//
//     fn set_header(mut self, name: String, value: String) {
//         self.headers.insert(name, value);
//     }
//
//     fn set_body(mut self, content: String) {
//         self.content = content;
//     }
//
//     fn send(mut self) {
//         let response = format!("HTTP/1.1 {}\r\nContent-Length: {}\r\n\r\n{}", http_code_to_string(self.status).unwrap(), self.content.len(), self.content);
//         self.stream.write(response.as_bytes()).unwrap();
//     }
// }
//
// impl FlorenceRequest for Request {
//     fn get_method(self) -> HttpMethod {
//         return self.method;
//     }
//
//     fn get_uri(self) -> String {
//         return self.uri;
//     }
//
//     fn get_body(self) -> String {
//         return self.body;
//     }
// }
//
// struct Route {
//     method: HttpMethod,
//     uri: String,
//     handler: fn(req: &dyn FlorenceRequest, res: &dyn FlorenceResponse)
// }
//
// pub struct Florence {
//     routes: Vec<Route>
// }
//
// impl Florence {
//     pub fn new() -> Self {
//         Florence {
//             routes: vec![]
//         }
//     }
// }
//
// impl Router for Florence {
//     fn route(mut self, method: HttpMethod, uri: String, handler: fn(&dyn FlorenceRequest, &dyn FlorenceResponse)) {
//         self.routes.push(Route{
//             method,
//             uri,
//             handler
//         });
//     }
//
//     fn get(mut self, uri: String, handler: fn(&dyn FlorenceRequest, &dyn FlorenceResponse)) {
//         self.route(HttpMethod::GET, uri, handler);
//     }
//
//     fn post(mut self, uri: String, handler: fn(&dyn FlorenceRequest, &dyn FlorenceResponse)) {
//         self.route(HttpMethod::POST, uri, handler);
//     }
//
//     fn put(mut self, uri: String, handler: fn(&dyn FlorenceRequest, &dyn FlorenceResponse)) {
//         self.route(HttpMethod::PUT, uri, handler);
//     }
//
//     fn delete(mut self, uri: String, handler: fn(&dyn FlorenceRequest, &dyn FlorenceResponse)) {
//         self.route(HttpMethod::DELETE, uri, handler);
//     }
//
//     fn patch(mut self, uri: String, handler: fn(&dyn FlorenceRequest, &dyn FlorenceResponse)) {
//         self.route(HttpMethod::PATCH, uri, handler);
//     }
//
//     fn match_route(self, req: &dyn FlorenceRequest) -> Result<Route, String> {
//         for route in self.routes {
//             let req_method = req.get_method();
//             if route.method == req_method && route.uri == req.get_uri() {
//                 return Ok(route);
//             }
//         }
//         Err("Unable to match request route".to_string())
//     }
// }
//
// impl Server for Florence {
//     fn start(self, port: u32) -> Result<(), String> {
//         return match TcpListener::bind(format!("127.0.0.1:{}", port)) {
//             Ok(listener) => {
//                 println!("Listening on port {}", port);
//                 for mut stream in listener.incoming() {
//                     let mut stream = stream.unwrap();
//                     println!("Connection established!");
//                     handle_connection(stream, Box::new(self));
//                 }
//                 Ok(())
//             }
//             Err(err) => {
//                 Err("Failed to start server".to_string())
//             }
//         }
//     }
// }
//
// fn http_code_to_string(code: u32) -> Result<&'static str, String> {
//     return match code {
//         200 => Ok("200 OK"),
//         201 => Ok("201 Created"),
//         202 => Ok("202 Accepted"),
//         204 => Ok("204 No Content"),
//         301 => Ok("301 Moved Permanently"),
//         302 => Ok("302 Found"),
//         303 => Ok("303 See Other"),
//         400 => Ok("400 Bad Request"),
//         401 => Ok("401 Unauthorized"),
//         403 => Ok("403 Forbidden"),
//         404 => Ok("404 Not Found"),
//         408 => Ok("408 Request Timeout"),
//         409 => Ok("409 Conflict"),
//         414 => Ok("414 URI Too Long"),
//         429 => Ok("429 Too Many Requests"),
//         500 => Ok("500 Internal Server Error"),
//         501 => Ok("501 Not Implemented"),
//         502 => Ok("502 Bad Gateway"),
//         503 => Ok("503 Service Unavailable"),
//         505 => Ok("505 HTTP Version Not Supported"),
//         _ => Err(format!("Status Code {} Not Implemented", code)),
//     }
// }
//
// fn raw_method_to_http_method(method: &str) -> Result<HttpMethod, String> {
//     return match method.to_uppercase().as_str() {
//         "GET" => Ok(HttpMethod::GET),
//         "HEAD" => Ok(HttpMethod::HEAD),
//         "POST" => Ok(HttpMethod::POST),
//         "PUT" => Ok(HttpMethod::PUT),
//         "DELETE" => Ok(HttpMethod::DELETE),
//         "CONNECT" => Ok(HttpMethod::CONNECT),
//         "OPTIONS" => Ok(HttpMethod::OPTIONS),
//         "TRACE" => Ok(HttpMethod::TRACE),
//         "PATCH" => Ok(HttpMethod::PATCH),
//         _ => Err(format!("Invalid request method"))
//     }
// }
//
// fn parse_request(stream: &mut TcpStream) -> Result<Request, String> {
//     let mut buffer = [0; 1024*12]; // 8k (apache max header size) + 4k start line
//     stream.read(&mut buffer).unwrap();
//     let content = String::from_utf8_lossy(&buffer[..]);
//     println!("{}", content);
//
//     let envelope: String = content.split("\r\n\r\n").nth(0).unwrap().to_owned();
//     let body: String = content.split("\r\n\r\n").nth(1).unwrap().to_owned();
//
//
//     let mut parts = envelope.split("\r\n");
//     let start_line: String = parts.next().unwrap().to_owned();
//
//     let mut start_line_parts = start_line.split(' ');
//
//     let method = raw_method_to_http_method(&start_line_parts.next().unwrap().to_owned());
//     if method.is_err() {
//         return Err(format!("Could not parse method from request"));
//     }
//
//     let uri:String = start_line_parts.next().unwrap().to_owned();
//     let mut headers: HashMap<String, String> = HashMap::new();
//     loop {
//         let next = parts.next();
//         if next.is_none() { break; }
//         let line:String = next.unwrap().to_owned();
//         let mut header_parts = line.split(": ");
//         headers.insert(header_parts.next().unwrap().to_owned(), header_parts.next().unwrap().to_owned());
//     }
//
//     return Ok(Request {
//         method: method.unwrap(),
//         uri,
//         headers,
//         body: body.trim_matches(char::from(0)).parse().unwrap(),
// //        stream
//     });
// }
//
// fn handle_connection(mut stream: TcpStream, router: Box<dyn Router>) {
//     let request = parse_request(&mut stream);
//     if request.is_err() {
//         let err_message = request.err().unwrap();
//         let response = format!("HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\n\r\n{}", err_message.len(), err_message);
//         stream.write(response.as_bytes()).unwrap();
//         stream.flush().unwrap();
//         return
//     } else {
//         println!("request obj {:?}", request.unwrap());
//     }
//     let route = router.match_route(&request.unwrap());
//     if route.is_err() {
//         let contents = "";
//         let response = format!("HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\n\r\n{}", contents.len(), contents);
//         stream.write(response.as_bytes()).unwrap();
//         stream.flush().unwrap();
//         return
//     }
//     let response = Response{stream, status: 200, content: "".to_string(), headers: Default::default() };
//     (route.unwrap().handler)(&request.unwrap(), &response);
// }
