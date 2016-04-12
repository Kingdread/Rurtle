extern crate rurtle;
mod support;

#[test]
fn test_smoke() {
    let mut turtle = support::turtle();
    turtle.forward(100.0);
}
