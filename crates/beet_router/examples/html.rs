use beet_router::prelude::*;


#[tokio::main]
async fn main() {
	let mut router = DefaultFileRouter::default();
	// usually its directly in src but test_site is a subdirectory
	// router.dst_dir = PathBuf::from("crates/beet_router/target/client")
	// 	.canonicalize()
	// 	.unwrap();"
	// router.dst_dir = "target/test_site".into();
	router.dst_dir = "target/test_site".into();
	router.templates_src = "target/test_site/rsx-templates.ron".into();
	beet_router::test_site::routes::collect_file_routes(&mut router);
	router.routes_to_html_files().await.unwrap();
}
