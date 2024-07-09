use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use hyper::{
    service::{make_service_fn, service_fn},
    Server
};

use tokio::sync::RwLock;
use dotenvy::dotenv;

use load_balancer::{handle, LBAlgorithm, LoadBalancer, LoadBalancerAlgorithm};
use load_balancer::constants::DEBUG_MODE;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let debug_mode = DEBUG_MODE.to_owned();
    if debug_mode {
        run_debug(debug_mode)
    } else {
        run_normal(debug_mode).await
    }
}

async fn run_normal(debug_mode: bool) {
    let load_balancer = create_load_balancer();

    let load_balancer = Arc::new(RwLock::new(load_balancer));

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 1337));
    
    let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
        let load_balancer = load_balancer.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(req, load_balancer.clone(), LoadBalancerAlgorithm::RoundRobin, debug_mode))) }
    }));
    
    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        println!("error: {}", e);
    }   
}

fn run_debug(debug_mode: bool) {
    let mut load_balancer = create_load_balancer();

    for i in [0,1,2,3,4,5,6,7,8,9,10] {
        let lba = if i%2 == 0 {LoadBalancerAlgorithm::RoundRobin} else {LoadBalancerAlgorithm::LeastConnections};
        let worker = load_balancer.next_worker(lba.clone(), debug_mode);
        println!("assigned_worker {} {:?}", worker, lba);
        println!("--------------------");
    }
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


