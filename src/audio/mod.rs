mod node;

pub enum Node {
    Oscillator(node::Oscillator),
    Destination(node::Destination),
}

pub enum Spliter{
    None,
    Only(u64),
    Equally,
}

pub enum Merger{
    None,
    Only(u64),
    Equally,
}

pub enum Connection {
    Pipeline(Vec<Connection>),
    Branch(Spliter, Vec<Connection>, Merger),
    LoopBack(Merger, Vec<Connection>, Spliter),
    Node(Node)
}