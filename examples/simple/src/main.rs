extern crate webview_rs;

use webview_rs::{Builder, Content};

fn main() {
    let mut webview = Builder::without_userdata()
        .set_title("Minimal webview example")
        .set_content(Content::Url("https://en.m.wikipedia.org/wiki/Main_Page"))
        .set_size(800, 600)
        .build()
        .unwrap();

    webview.run(true);
}
