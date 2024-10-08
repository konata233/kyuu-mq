#[derive(Debug)]
pub enum RoutingKey {
    Direct([String; 4]),
    Topic([String; 4]),
    Fanout([String; 4]),
}