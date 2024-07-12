use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use hyper::{
    service::{make_service_fn, service_fn},
    Server
};

use tokio::sync::RwLock;
use dotenvy::dotenv;

use rand::thread_rng;
use rand::seq::SliceRandom;

use load_balancer::handle;
use load_balancer::lb_service::{NextWorker, LoadBalancer, LoadBalancerAlgorithm};
use load_balancer::constants::{DEBUG_MODE, REMOVE_CONN};

const CHOICE0: [isize; 15] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const CHOICE1: [isize; 15] = [0, -1, 0, -1, 0, 0, -1, -1, 0, -1, 0, 0, 0, -1, 0];

#[tokio::main]
async fn main() {
    dotenv().ok();
    let debug_mode = DEBUG_MODE.to_owned();
    let remove_conn: bool = REMOVE_CONN.to_owned();
     if debug_mode {
        run_debug(remove_conn)
    } else {
        run_normal().await
    }
}

async fn run_normal() {
    let load_balancer = create_load_balancer();

    let load_balancer = Arc::new(RwLock::new(load_balancer));

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 1337));
    
    let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
        let load_balancer = load_balancer.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(req, load_balancer.clone(), LoadBalancerAlgorithm::RoundRobin))) }
    }));
    
    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        println!("error: {}", e);
    }   
}

fn run_debug(remove_conn: bool) {
    let mut load_balancer = create_load_balancer();
    let choice = if remove_conn {CHOICE1} else {CHOICE0};

    println!("");
    for i in [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15] {
        let lba = if i%2 == 0 {LoadBalancerAlgorithm::RoundRobin} else {LoadBalancerAlgorithm::LeastConnections};
        let worker = load_balancer.next_worker(lba.clone());
        let conn01 = load_balancer.get_conn(&worker);
        let value = update_value(choice);
        if value < 0 {
            load_balancer.dec_conn(&worker);
        }
        let conn02 = load_balancer.get_conn(&worker);
        println!("worker: {} conn: {} update: {value:>2} updated_conn: {} {:?}", worker, conn01, conn02, lba);
        println!("-----------------------------");
    }
    println!("");
}

fn create_load_balancer() -> LoadBalancer {
    let worker_hosts = vec![
        "http://localhost:3000".to_string(),
        "http://localhost:3001".to_string(),
        "http://localhost:3002".to_string(),            
        "http://localhost:3003".to_string(),
        "http://localhost:3004".to_string(),
    ];

    LoadBalancer::new(worker_hosts).unwrap()
}

fn update_value(choice: [isize; 15]) -> isize {
    let mut rng = thread_rng();
    *choice.choose(&mut rng).unwrap()
}


