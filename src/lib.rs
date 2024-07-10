use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

use hyper::{
    client::ResponseFuture,
    Body, Request, Uri, Response
};

use lb_service::{LoadBalancer, LoadBalancerAlgorithm, NextWorker};

pub mod constants;
pub mod lb_service;

pub async fn handle(req: Request<Body>,
    load_balancer: Arc<RwLock<LoadBalancer>>,
    lba: LoadBalancerAlgorithm) -> Result<Response<Body>, hyper::Error> {
    let mut load_balancer = load_balancer.write().await;
    let result = forward_request(&mut load_balancer, req, lba).await;
    result
}

pub fn forward_request(lb: &mut LoadBalancer, 
    req: Request<Body>, 
    lba: LoadBalancerAlgorithm) -> ResponseFuture {
    let mut worker_uri = lb.next_worker(lba).to_owned();

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

    lb.client.request(new_req)
}
