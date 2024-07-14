use std::collections::HashMap;

use hyper::Client;

type Worker = String;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum LoadBalancerError {
    EmptyWorkerList,
    #[default]
    UnexpectedError
}

pub fn default_worker() -> Worker {
    "".to_owned()
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

    fn update_conn(&mut self, worker: &Worker, uvalue: isize) {
        if let Some(conn) = self.worker_conn_map.get_mut(worker) {
            *conn += uvalue;
        }
    }

    fn inc_conn(&mut self, worker: &Worker) {
        self.update_conn(worker, 1);
    }

    pub fn dec_conn(&mut self, worker: &Worker) {
        self.update_conn(worker, -1);
    }

    fn worker(&self, index: usize) -> Worker {
        self.worker_hosts[index].clone()
    }

    fn host_len(&self) -> usize {
        self.worker_hosts.len()
    }

    pub fn get_conn(&self, worker: &Worker) -> isize {
        *self.worker_conn_map.get(worker).unwrap()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum LoadBalancerAlgorithm {
    #[default]
    RoundRobin,
    LeastConnections
}

pub struct LoadBalancer {
    pub client: Client<hyper::client::HttpConnector>,
    pub worker_hosts: Vec<Worker>,
    pub next_worker: usize,
    pub worker_conn_map: HashMap<Worker, isize>
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
fn next_worker_round_robin(lb: &mut LoadBalancer, len: usize) -> Worker {
    let worker = lb.worker(lb.next_worker);
    update_state(lb, &worker, lb.next_worker, len);
    worker
}

fn next_worker_least_connections(lb: &mut LoadBalancer, len: usize) -> Worker {
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
    update_state(lb, &worker, worker_index, len);
    worker
}

fn update_state(lb: &mut LoadBalancer, worker: &Worker, worker_index: usize, len: usize) {
    lb.inc_conn(worker);
    lb.next_worker = (worker_index+1) % len;
}
