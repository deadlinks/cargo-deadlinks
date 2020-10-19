/// Foo function
///
/// Has something to do with [bar](./fn.bar.html).
pub fn foo() {}

/// Bar function
pub fn bar() {}

/// [Correct link](crate::bar)
pub struct Tmp {}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
