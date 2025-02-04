use beet_rsx_parser::prelude::*;
use proc_macro::TokenStream;

/// Demonstrates how to select a different reactive runtime
/// this is quite unsophisticated at the moment, we can work on a nicer
/// way to expose it to library authors
#[allow(unused_mut)]
fn idents() -> RsxIdents {
	let mut idents = RsxIdents::default();
	#[cfg(not(feature = "signals"))]
	{
		idents.effect = syn::parse_quote!(beet::rsx::string_rsx::StringRsx);
	}
	idents
}



/// This macro expands to an [RsxNode](beet_rsx::prelude::RsxNode).
///
/// The type of node is determied by the feature flags, current options are:
/// - [`StringRsx`](beet_rsx::rsx::StringRsx)
/// ```
/// # use beet::prelude::*;
/// let tree = rsx! {<div> the value is {3}</div>};
/// assert_eq!(tree.nodes.len(), 1);
///
/// ```
///
#[proc_macro]
pub fn rsx(tokens: TokenStream) -> TokenStream {
	RstmlToRsx {
		// perhaps we can feature gate this if it proves expensive
		hash_location: true,
		idents: idents(),
		..Default::default()
	}
	.map_tokens(tokens.into())
	.into()
}

/// Mostly used for testing,
/// this macro expands to a Vec<RsxTemplateNode>, it is used for
/// things like hot reloading.
#[proc_macro]
pub fn rsx_template(tokens: TokenStream) -> TokenStream {
	RstmlToRsxTemplate::default()
		.map_tokens(tokens.into())
		.into()
}
