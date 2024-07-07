use std::str::FromStr;

use hyper::{
    client::ResponseFuture,
    Body, Client, Request, Uri
};


#[derive(Debug, Clone)]
pub struct Worker {
    server: String,
    no_conn: isize
}

impl Worker {
    pub fn new(server: String) -> Self {
        Self {
            server,
            no_conn: 0
        }
    }

    pub fn update(&mut self, uvalue: isize) {
        self.no_conn += uvalue;
    }

    pub fn inc(&mut self) {
        self.update(1);
    }

    pub fn dec(&mut self) {
        self.update(-1);
    }

    pub fn server(&self) -> String {
        self.server.clone()
    }
}

impl Default for Worker {
    fn default() -> Self {
        Worker::new("".to_string())
    }
}

#[derive(Debug, Clone)]
pub enum LoadBalancerAlgorithm {
    RoundRobin,
    LeastConnections
}
#[derive(Debug)]
pub enum LoadBalancerError {
    EmptyWorkerList,
    UnexpectedError
}
pub struct LoadBalancer {
    pub client: Client<hyper::client::HttpConnector>,
    pub worker_hosts: Vec<Worker>,
    pub current_worker: usize,
    pub inuse_worker: Option<usize>
}

impl LoadBalancer {
    pub fn new(svec: Vec<String>) -> Result<Self, LoadBalancerError> {
        if svec.is_empty() {
            return Err(LoadBalancerError::EmptyWorkerList);
        }

        let mut worker_hosts = Vec::new();
        for server in svec {
            worker_hosts.push(Worker::new(server))
        };

        Ok(Self {client: Client::new(), 
                 worker_hosts, 
                 current_worker: 0,
                 inuse_worker: None})
    }

    pub fn dec_conn(&mut self) {
        match self.inuse_worker {
            Some(index) => {let server = &mut self.worker_hosts[index];
                                    server.dec();
                                    self.inuse_worker = None;},
            None => return
        }
    }

    pub fn forward_request(&mut self, req: Request<Body>, lba: LoadBalancerAlgorithm) -> ResponseFuture {
        let mut worker_uri = self.next_server(lba).to_owned();

        // Extract the path and query from the original request
        if let Some(path_and_query) = req.uri().path_and_query() {
            worker_uri.push_str(path_and_query.as_str());
        }

        // Create a new URI from the worker URI
        let new_uri = Uri::from_str(&worker_uri).unwrap();

        // Extract the headers from the original request
        let headers = req.headers().clone();

        // Clone the original request's headers and method
        let mut new_req = Request::builder()
            .method(req.method())
            .uri(new_uri)
            .body(req.into_body())
            .expect("request builder");

        // Copy headers from the original request
        for (key, value) in headers.iter() {
            new_req.headers_mut().insert(key, value.clone());
        }

        self.client.request(new_req)
    }
}

pub trait LBAlgorithm {
    fn next_server(&mut self, lba: LoadBalancerAlgorithm) -> String;
}

impl LBAlgorithm for LoadBalancer {
    fn next_server(&mut self, lba: LoadBalancerAlgorithm) -> String {
        let len:usize = self.worker_hosts.len();
        match lba {
            LoadBalancerAlgorithm::RoundRobin => next_server_round_robin(self, len),
            LoadBalancerAlgorithm::LeastConnections => next_server_least_connections(self, len)
        }
    }
}

pub fn next_server_round_robin(lb: &mut LoadBalancer, len: usize) -> String {
    let server = &mut lb.worker_hosts[lb.current_worker];
    server.inc();
    lb.inuse_worker = Some(lb.current_worker);
    lb.current_worker = (lb.current_worker + 1) % len;
    server.server()
}

pub fn next_server_least_connections(lb: &mut LoadBalancer, len: usize) -> String {
    let mut first = true;
    let mut server = &mut Worker::default();
    let mut index : usize = 0;
    for (i, s) in lb.worker_hosts.iter_mut().enumerate() {
        if first || s.no_conn < server.no_conn {
            server = s;
            index = i
        }
        first = false;
    }
    server.inc();
    lb.inuse_worker = Some(index);
    lb.current_worker = (index+1) % len;
    server.server()
}

