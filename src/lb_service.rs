use std::collections::HashMap;

use hyper::Client;

type Worker = String;

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

pub fn default_worker() -> Worker {
    "".to_owned()
}

pub struct LoadBalancer {
    pub client: Client<hyper::client::HttpConnector>,
    pub worker_hosts: Vec<Worker>,
    pub next_worker: usize,
    pub worker_conn_map: HashMap<Worker, isize>
}

impl LoadBalancer {
    pub fn new(wvec: Vec<Worker>) -> Result<Self, LoadBalancerError> {
        if wvec.is_empty() {
            return Err(LoadBalancerError::EmptyWorkerList);
        }

        let mut worker_hosts = Vec::new();
        let mut worker_conn_map = HashMap::new();

        for worker in wvec {
            worker_hosts.push(worker.clone());
            worker_conn_map.insert(worker, 0);
        };

        Ok(Self{
            client: Client::new(),
            worker_hosts,
            worker_conn_map,
            next_worker: 0
        })
    }

    pub fn update(&mut self, worker: &Worker, uvalue: isize) {
        if let Some(conn) = self.worker_conn_map.get_mut(worker) {
            *conn += uvalue;
        }
    }

    pub fn inc(&mut self, worker: &Worker) {
        self.update(worker, 1);
    }

    pub fn dec(&mut self, worker: &Worker) {
        self.update(worker, -1);
    }

    pub fn worker(&self, index: usize) -> Worker {
        self.worker_hosts[index].clone()
    }

    pub fn host_len(&self) -> usize {
        self.worker_hosts.len()
    }

    pub fn get_conn(&self, worker: &Worker) -> isize {
        *self.worker_conn_map.get(worker).unwrap()
    }
}

pub trait NextWorker {
    fn next_worker(&mut self, lba: LoadBalancerAlgorithm) -> Worker;
}

impl NextWorker for LoadBalancer {
    fn next_worker(&mut self, lba: LoadBalancerAlgorithm) -> Worker {
        let len = self.host_len();
        match lba {
            LoadBalancerAlgorithm::RoundRobin       => next_worker_round_robin(self, len),
            LoadBalancerAlgorithm::LeastConnections => next_worker_least_connections(self, len)
        }
    }
}
pub fn next_worker_round_robin(lb: &mut LoadBalancer, len: usize) -> Worker {
    let worker = lb.worker(lb.next_worker);
    lb.inc(&worker);
    lb.next_worker = (lb.next_worker+1) % len;
    worker
}

pub fn next_worker_least_connections(lb: &mut LoadBalancer, len: usize) -> Worker {
    let mut first = true;
    let mut worker = default_worker();
    let mut conn: isize = 0;
    let mut worker_index: usize = 0;
    for (i, w) in lb.worker_hosts.iter().enumerate() {
        let wconn = lb.get_conn(w);
        if first || wconn < conn {
            conn = wconn;
            worker_index = i;
            worker = lb.worker(worker_index);
        }
        first = false;
    }
    lb.inc(&worker);
    lb.next_worker = (worker_index+1) % len;
    worker
}

