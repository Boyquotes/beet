//! 🥁🥁🥁 This file has been auto generated by the Beet router.
//! 🥁🥁🥁 Any changes will be overridden if the file is regenerated.
pub mod contributing;
pub mod index;
pub mod docs;
use crate::prelude::*;
pub fn collect_file_routes(router: &mut crate::DefaultFileRouter) {
    router.add_route((RouteInfo::new("/contributing", "get"), contributing::get));
    router.add_route((RouteInfo::new("/", "get"), index::get));
    docs::collect_file_routes(router);
}
