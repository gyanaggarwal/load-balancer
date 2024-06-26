use crate::domain::load_balancer::LoadBalancer;

pub struct Server {
    server: String,
    no_conn: usize
}

impl Default for Server {
    fn default() -> Self {
        Server {
            server: "".to_owned(),
            no_conn: 0
        }
    }
}

impl Server {
    fn update(&mut self, uvalue: usize) {
        self.no_conn += uvalue;
    }
}
pub struct LeastConnectionsLoadBalancer {
    servers: Vec<Server>
}

impl LoadBalancer for LeastConnectionsLoadBalancer {
    fn next_server(&mut self) -> String {
        let mut first = true;
        let mut server = &mut Server::default();
        for s in self.servers.iter_mut() {
            if first || s.no_conn < server.no_conn {
                server = s
            }
            first = false;
        }
        server.update(1);
        server.server.clone()
    }
}
