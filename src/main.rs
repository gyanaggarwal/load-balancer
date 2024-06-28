use std::sync::{Arc, RwLock};

use load_balancer::services::{round_robin::RoundRobinLoadBalancer, 
                              least_connections::LeastConnectionsLoadBalancer};
use load_balancer::domain::load_balancer::LoadBalancer;

fn main() {
    let mut slist0 = Vec::new();

    slist0.push("localhost:8001".to_owned());
    slist0.push("localhost:8002".to_owned());
    slist0.push("localhost:8003".to_owned());
    slist0.push("localhost:8004".to_owned());
    slist0.push("localhost:8005".to_owned());
    
    let slist1 = slist0.clone();

    let rr_lb = RoundRobinLoadBalancer::new(slist0);
    let lc_lb = LeastConnectionsLoadBalancer::new(slist1);
    let arr_lb = Arc::new(RwLock::new(rr_lb));
    let alc_lb = Arc::new(RwLock::new(lc_lb));

    check_loadbalancer(arr_lb, "RR");
    check_loadbalancer(alc_lb, "LC");
}

fn check_loadbalancer(alb: Arc<RwLock<impl LoadBalancer>>, name: &str) {
    let mut lb0 = alb.write().unwrap();

    for i in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]{
        let server = lb0.next_server();
        println!("server {} {} {}", name, i, server);
    }
}
