extern crate cc;
extern crate pkg_config;

use std::env;

fn main() {
    let mut build = cc::Build::new();

    build.flag_if_supported("-std=c11");
    build.include("src/webview");
    build.file("src/webview_wrapper.c");

    if env::var("DEBUG").is_err() {
        build.define("NDEBUG", None);
    } else {
        build.define("DEBUG", None);
    }

    let target = env::var("TARGET").unwrap();
    if target.contains("windows") {
        build.define("WEBVIEW_API", None);
        for &lib in &["ole32", "comctl32", "oleaut32", "uuid", "gdi32"] {
            println!("cargo:rustc-link-lib={}", lib);
        }
    } else if target.contains("linux") || target.contains("bsd") {
        let webkit = pkg_config::Config::new()
            .atleast_version("2.8")
            .probe("webkit2gtk-4.0")
            .unwrap();

        for path in webkit.include_paths {
            build.include(path);
        }

        build.define("WEBVIEW_GTK", None);
    } else if target.contains("apple") {
        build.define("WEBVIEW_COCOA", None);
        build.flag("-x");
        build.flag("objective-c");

        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=WebKit");
    } else {
        panic!("build not supported for this target");
    }

    build.compile("webview")
}