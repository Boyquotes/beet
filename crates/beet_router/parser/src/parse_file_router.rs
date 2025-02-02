pub use crate::prelude::*;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use sweet::prelude::*;
use syn::File;

/// Parse a 'pages' dir, collecting all the routes,
/// and create a file called `routes.rs` which contains
/// a ServerRoutes struct with all the routes.
#[derive(Debug, Parser)]
pub struct ParseFileRouter {
	/// location of the file router relative to the src directory
	#[arg(long, default_value = "file_router.rs")]
	pub file_router_path: PathBuf,
	/// Optionally specify additional tokens to be added to the top of the file.
	#[arg(long)]
	pub file_router_tokens: Option<String>,
	/// Identifier for the router. The router must have
	/// where T can be a type or trait for each route on the site.
	#[arg(long, default_value = "beet::router::DefaultFileRouter")]
	pub file_router_ident: String,
	/// location of the src directory,
	#[arg(long, default_value = "src")]
	pub src: PathBuf,
	/// location of the pages directory relative to the src directory
	#[arg(long, default_value = "pages")]
	pub pages_dir: PathBuf,
}

impl Default for ParseFileRouter {
	fn default() -> Self { clap::Parser::parse_from(&[""]) }
}

impl ParseFileRouter {
	fn src_dir(&self) -> &PathBuf { &self.src }
	fn pages_dir(&self) -> PathBuf { self.src_dir().join(&self.pages_dir) }
	fn file_router_path(&self) -> PathBuf {
		self.src_dir().join(&self.file_router_path)
	}

	pub fn write_to_file(&self) -> Result<()> {
		let routes = self.build_string()?;
		let routes_file = self.file_router_path();
		std::fs::write(&routes_file, routes)?;
		Ok(())
	}

	pub fn build_string(&self) -> Result<String> {
		let page_routes = ReadDir::files_recursive(self.pages_dir())?
			.iter()
			.map(|path| ParseRouteFile::parse(&path.canonicalize()?))
			.collect::<Result<Vec<_>>>()?
			.into_iter()
			.flatten();

		let router_ident = &self.file_router_ident;
		let router_ident: syn::Type = syn::parse_str(router_ident).unwrap();

		// let ident_tokens =
		// let ident = syn::Ident::new(router_ident, Span::call_site());
		// let syn_path: syn::Path = parse_quote!(#router_ident);
		// let syn_path = syn_path.to_token_stream();

		let prefix_tokens: File = self
			.file_router_tokens
			.as_ref()
			.map(|tokens| syn::parse_str(tokens))
			.unwrap_or_else(|| {
				Ok(syn::parse_quote! {
					//! 🥁🥁🥁 This file has been auto generated by the Beet router.
					//! 🥁🥁🥁 Any changes will be overridden if the file is regenerated.
					use beet::prelude::*;
				})
			})?;

		let file: File = syn::parse_quote! {
			#prefix_tokens
			pub fn collect_file_routes(router: &mut #router_ident) {
				#(
					router.add_route(#page_routes);
				)*
			}
		};
		let file = prettyplease::unparse(&file);

		Ok(file)
	}


	pub fn build_and_write(&self) -> Result<()> {
		let data = self.build_string()?;
		FsExt::write(self.file_router_path(), &data)?;
		Ok(())
	}
}


#[cfg(test)]
mod test {
	// indirectly testd via build_test_site.rs
}
