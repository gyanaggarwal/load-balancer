use load_balancer::services::{round_robin::RoundRobinLoadBalancer, 
                              least_connections::LeastConnectionsLoadBalancer};

fn main() {
    let mut slist0 = Vec::new();

    slist0.push("localhost:8001".to_owned());
    slist0.push("localhost:8002".to_owned());
    slist0.push("localhost:8003".to_owned());
    slist0.push("localhost:8004".to_owned());
    slist0.push("localhost:8005".to_owned());
    
    let slist1 = slist0.clone();

    let _ = RoundRobinLoadBalancer::new(slist0);
    let _ = LeastConnectionsLoadBalancer::new(slist1);
    
    println!("Hello, world!");
}
