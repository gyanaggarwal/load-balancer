pub trait LoadBalancer {
    fn next_server(&mut self) -> String;
}