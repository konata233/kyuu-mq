use crate::test::test::test;
// always remember that the last value of RoutingKey is the name of the Queue.
pub mod test;
pub mod mq;

fn main() {
    test();
}
