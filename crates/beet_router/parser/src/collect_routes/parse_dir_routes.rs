pub use crate::prelude::*;
use anyhow::Result;
use quote::quote;
use std::path::Path;
use sweet::prelude::*;
use syn::File;




pub struct ParseDirRoutes;


impl ParseDirRoutes {
	pub fn build_string(config: &CollectRoutes, path: &Path) -> Result<String> {
		let routes_dir_name = config.routes_dir.to_string_lossy();
		let page_routes = ReadDir::files(path)?
			.into_iter()
			.map(|path| ParseFileRoutes::parse(&routes_dir_name, path))
			.collect::<Result<Vec<_>>>()?
			.into_iter()
			.flatten();

		let router_ident = &config.file_router_ident;
		let router_ident: syn::Type = syn::parse_str(router_ident)?;

		// let ident_tokens =
		// let ident = syn::Ident::new(router_ident, Span::call_site());
		// let syn_path: syn::Path = parse_quote!(#router_ident);
		// let syn_path = syn_path.to_token_stream();

		let prefix_tokens: File = config
			.file_router_tokens
			.as_ref()
			.map(|tokens| syn::parse_str(tokens))
			.unwrap_or_else(|| {
				Ok(syn::parse_quote! {
					use beet::prelude::*;
				})
			})?;

		let dir_idents = ReadDir::dirs(path)?
			.into_iter()
			.map(|path| {
				let name = path
					.file_stem()
					.expect("dir has no stem")
					.to_string_lossy();
				syn::Ident::new(&name, proc_macro2::Span::call_site())
			})
			.collect::<Vec<_>>();
		let include_files = ReadDir::files(path)?
			.into_iter()
			.filter(|path| {
				path.extension().map(|ext| ext == "rs").unwrap_or(false)
					&& path.file_name().unwrap() != "mod.rs"
			})
			.map(|path| {
				let name = path
					.file_stem()
					.expect("file has no stem")
					.to_string_lossy();
				let ident =
					syn::Ident::new(&name, proc_macro2::Span::call_site());
				quote! {pub mod #ident; }
			});
		let include_dirs =
			dir_idents.iter().map(|ident| quote! {pub mod #ident;});
		let collect_dirs = dir_idents.iter().map(|ident| {
			quote! {	#ident::collect_file_routes(router);}
		});

		let file: File = syn::parse_quote! {
			//! 🥁🥁🥁 This file has been auto generated by the Beet router.
			//! 🥁🥁🥁 Any changes will be overridden if the file is regenerated.
			// routes
			#(#include_files)*
			// Sub routes
			#(#include_dirs)*
			#prefix_tokens
			pub fn collect_file_routes(router: &mut #router_ident) {
				#(router.add_route(#page_routes);)*
				#(#collect_dirs)*
			}
		};
		let file = prettyplease::unparse(&file);

		Ok(file)
	}
}
