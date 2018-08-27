extern crate webview_rs;

use webview_rs::Builder;
use webview_rs::Content;
use webview_rs::WebviewHandle;

fn main() {
    let mut webview: WebviewHandle<()> = Builder::new()
        .set_title("Minimal webview example")
        .set_content(Content::Url("https://en.m.wikipedia.org/wiki/Main_Page"))
        .set_size(800, 600)
        .set_resizable(true)
        .set_debug(true)
        .build()
        .unwrap();

    webview.run(true);
}
