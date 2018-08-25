extern crate webview_rs;

use webview_rs::WebviewBuilder;
use webview_rs::Content;
use webview_rs::Webview;

fn main() {
    let webview: Webview<()> = WebviewBuilder::new()
        .set_title("Minimal webview example")
        .set_content(Content::Url("https://en.m.wikipedia.org/wiki/Main_Page"))
        .set_width(800)
        .set_height(600)
        .set_resizable(true)
        .set_debug(true)
        .build()
        .unwrap();

    webview.run(true);
}
