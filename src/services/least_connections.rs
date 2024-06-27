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
    fn new(server: String) -> Self {
        Self {
            server,
            no_conn: 0
        }
    }

    fn update(&mut self, uvalue: usize) {
        self.no_conn += uvalue;
    }
}
pub struct LeastConnectionsLoadBalancer {
    servers: Vec<Server>
}

impl LeastConnectionsLoadBalancer {
    pub fn new (slist: Vec<String>) -> Self {
        let mut servers = Vec::new();
        for server in slist {
            servers.push(Server::new(server))
        }

        Self {
            servers
        }
    }
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
