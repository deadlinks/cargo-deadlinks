//! [Non-ascii link](#†)
//!
//! <div id="†">Some text</div>

/// Foo function
///
/// Has something to do with [bar](./fn.bar.html).
pub fn foo() {}

// not sure how to do this with intra-doc links, but this is close enough to test deadlinks.
/// Bar function that links to [S](./inner/struct.S.html#associatedtype.Item)
pub fn bar() {}

/// [Correct link](crate::bar)
pub struct Tmp {}

mod inner {
	// This generates a link from inner/S to the crate-level S
	pub struct S;

	impl Iterator for S {
		type Item = ();
		fn next(&mut self) -> Option<()> {
			None
		}
	}
}

pub use inner::S;
