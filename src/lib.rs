use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

use hyper::{
    client::ResponseFuture,
    Body, Client, Request, Uri, Response
};

pub mod constants;

#[derive(Debug, Clone)]
pub struct Worker {
    worker: String,
    no_conn: isize
}

impl Worker {
    pub fn new(worker: String) -> Self {
        Self {
            worker,
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

    pub fn worker(&self) -> String {
        self.worker.clone()
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
    pub next_worker: usize,
    pub inuse_worker: Option<usize>
}

impl LoadBalancer {
    pub fn new(svec: Vec<String>) -> Result<Self, LoadBalancerError> {
        if svec.is_empty() {
            return Err(LoadBalancerError::EmptyWorkerList);
        }

        let mut worker_hosts = Vec::new();
        for worker in svec {
            worker_hosts.push(Worker::new(worker))
        };

        Ok(Self {client: Client::new(), 
                 worker_hosts, 
                 next_worker: 0,
                 inuse_worker: None})
    }

    pub fn dec_conn(&mut self) {
        match self.inuse_worker {
            Some(index) => {let worker = &mut self.worker_hosts[index];
                                    worker.dec();
                                    self.inuse_worker = None;},
            None => return
        }
    }

    pub fn forward_request(&mut self, req: Request<Body>, lba: LoadBalancerAlgorithm, debug_mode: bool) -> ResponseFuture {
        let mut worker_uri = self.next_worker(lba, debug_mode).to_owned();

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
    fn next_worker(&mut self, lba: LoadBalancerAlgorithm, debug_mode: bool) -> String;
}

impl LBAlgorithm for LoadBalancer {
    fn next_worker(&mut self, lba: LoadBalancerAlgorithm, debug_mode: bool) -> String {
        let len:usize = self.worker_hosts.len();
        match lba {
            LoadBalancerAlgorithm::RoundRobin => next_worker_round_robin(self, debug_mode, len),
            LoadBalancerAlgorithm::LeastConnections => next_worker_least_connections(self, debug_mode, len)
        }
    }
}

pub fn next_worker_round_robin(lb: &mut LoadBalancer, debug_mode: bool, len: usize) -> String {
    let worker = &mut lb.worker_hosts[lb.next_worker];
    worker.inc();
    lb.inuse_worker = Some(lb.next_worker);
    lb.next_worker = (lb.next_worker + 1) % len;
    if debug_mode {
        println!("conn {}, assigned_worker {}, next_worker {}", worker.no_conn, worker.worker(), lb.next_worker);
    }
    worker.worker()
}

pub fn next_worker_least_connections(lb: &mut LoadBalancer, debug_mode: bool, len: usize) -> String {
    let mut first = true;
    let mut worker = &mut Worker::default();
    let mut index : usize = 0;
    for (i, s) in lb.worker_hosts.iter_mut().enumerate() {
        if first || s.no_conn < worker.no_conn {
            worker = s;
            index = i
        }
        first = false;
    }
    worker.inc();
    lb.inuse_worker = Some(index);
    lb.next_worker = (index+1) % len;
    if debug_mode {
        println!("conn {}, assigned_worker {}, next_worker {}", worker.no_conn, worker.worker(), lb.next_worker);
    }
    worker.worker()
}

pub async fn handle(req: Request<Body>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
    lba: LoadBalancerAlgorithm,
    debug_mode: bool) -> Result<Response<Body>, hyper::Error> {
    let mut load_balancer = load_balancer.write().await;
    let result = load_balancer.forward_request(req, lba, debug_mode).await;
    load_balancer.dec_conn();
    result
}
