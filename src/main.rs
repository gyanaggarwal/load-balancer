use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server
};

use tokio::sync::RwLock;
    
use load_balancer::{LoadBalancer, LoadBalancerAlgorithm};

async fn handle(
    req: Request<Body>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
    lba:&LoadBalancerAlgorithm) -> Result<Response<Body>, hyper::Error> {
    let mut load_balancer = load_balancer.write().await;
    let result = load_balancer.forward_request(req, lba).await;
    load_balancer.dec_conn();
    result
}
    
#[tokio::main]
async fn main() {
    let worker_hosts = vec![
        "http://localhost:3000".to_string(),
        "http://localhost:3001".to_string(),
        "http://localhost:3002".to_string(),            
        "http://localhost:3003".to_string(),
        "http://localhost:3004".to_string(),
    ];
    
    let load_balancer = Arc::new(RwLock::new(
        LoadBalancer::new(worker_hosts).expect("failed to create load balancer"),
    ));

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 1337));
    
    let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
        let load_balancer = load_balancer.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(req, load_balancer.clone(), &LoadBalancerAlgorithm::RoundRobin))) }
    }));
    
    if let Err(e) = server.await {
        println!("error: {}", e);
    }          
}

