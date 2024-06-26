use crate::domain::load_balancer::LoadBalancer;

pub struct RoundRobinLoadBalancer {
   servers: Vec<String>,
   current_index: usize 
}

impl LoadBalancer for RoundRobinLoadBalancer {
    fn next_server(&mut self) -> String {
        let server = self.servers[self.current_index].clone();
        self.current_index = (self.current_index + 1) % self.servers.len();
        server
    }
}