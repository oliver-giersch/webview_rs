extern crate rand;
extern crate webview_rs;

use std::thread;
use std::time;

use webview_rs::{Builder, Content};

const THREADS: usize = 16;
const BLACK: [u8; 4] = [0, 0, 0, 0];

const HTML: &'static str = include_str!("../assets/index.html");
const CSS: &'static str = include_str!("../assets/styles.css");

fn main() {
    let mut webview = Builder::without_userdata()
        .set_title("Stresstest")
        .set_content(Content::Html(HTML))
        .set_size(800, 600)
        .set_debug(true)
        .build()
        .unwrap();

    webview.inject_css(CSS).expect("could not inject css");

    for id in 0..THREADS {
        let handle = webview.thread_handle();
        thread::spawn(move || {
            let id_string = id.to_string();

            loop {
                thread::sleep(time::Duration::from_millis(50));

                let random: u8 = rand::random();
                let color = match random % 4 {
                    0 => "'red'",
                    1 => "'green'",
                    2 => "'blue'",
                    3 => "'yellow'",
                    _ => unreachable!(),
                };

                let result = handle.try_dispatch(|webview, _| {
                    webview.eval_fn("setColor", &[&id_string, color]);
                });

                if result.is_err() {
                    break;
                }
            }
        });
    }

    webview.run(true);
}
