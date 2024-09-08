/*! An easier way to mask code blocks than commenting them out.
 *
 * Due to [rust#54727](https://github.com/rust-lang/rust/issues/54727), it can
 * not be used on `{ }` blocks yet.
 */
use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

/** Swallow the body of the `fn` it's attached to.
 *
 * ```rust
 * #[hollow::hollow]
 * fn function_to_swallow() {
 *     panic!("this panic! should be swallowed by hollow");
 * }
 *
 * function_to_swallow()
 * ```
 *
 * When its return type does not `impl Default`, or a value other than the
 * default is desired:
 *
 * ```rust
 * struct NoDefault {
 *     value: u64,
 * }
 *
 * #[hollow::hollow(value = NoDefault { value: 42 })]
 * fn custom_return() -> NoDefault {
 *     let a = 4;
 *     let b = 9;
 *     let c = 8;
 *     let d = 1;
 *     NoDefault { value: a * b + c / d }
 * }
 *
 * assert_eq!(42, custom_return().value);
 * ```
 */
#[proc_macro_attribute]
pub fn hollow(attr: TokenStream, item: TokenStream) -> TokenStream {
	let body_tokens = if attr.is_empty() {
		"Default::default()".parse().unwrap()
	} else {
		let mut iter = attr.into_iter();
		let Some(TokenTree::Ident(next)) = iter.next() else {
			panic!("invalid attr argument");
		};
		assert_eq!("value", &next.to_string());
		let Some(TokenTree::Punct(next)) = iter.next() else {
			panic!("invalid attr argument");
		};
		assert_eq!('=', next.as_char());
		TokenStream::from_iter(iter)
	};

	let mut tokens = Vec::new();

	/* Items to be hollowed are basically fns; they start with a few Idents,
	 * optionally a <>-delimited generics Group, then a ()- delimited
	 * parameters Group, then an optional Ident set of -> ReturnType,
	 * then a {}-delimited body Group.
	 *
	 * To be hollowed is the body; everything before it should be preserved.
	 *
	 * For fns with a return type, if no attr argument is given, we insert a
	 * `Default::default()` as the body; otherwise, insert the attr argument.
	 */
	for token in item.into_iter() {
		match token {
			TokenTree::Group(group) => match group.delimiter() {
				Delimiter::Brace => break,
				_ => tokens.push(TokenTree::Group(group)),
			},
			other => tokens.push(other),
		}
	}

	tokens.push(TokenTree::Group(Group::new(Delimiter::Brace, body_tokens)));

	TokenStream::from_iter(tokens)
}
