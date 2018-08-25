extern crate webview_rs;

use std::sync::{Arc, Mutex};
use std::time;
use std::thread;

use webview_rs::{Webview, WebviewBuilder, Content, Userdata};

static HTML_DATA: &'static str = include_str!("../index.html");

#[derive(Default)]
struct Timer {
    ticks: Mutex<usize>,
}

impl Timer {
    fn get(&self) -> usize {
        let lock = self.ticks.lock().unwrap();
        *lock
    }

    fn set(&self, ticks: usize) {
        let mut lock = self.ticks.lock().unwrap();
        *lock = ticks;
    }

    fn incr(&self) {
        let mut lock = self.ticks.lock().unwrap();
        *lock += 1;
    }

    fn render(&self, webview: &Webview<Arc<Timer>>) {
        let ticks = self.get();
        webview.eval_fn("updateTicks", &["ticks"]);
    }
}

fn main() {
    let timer: Arc<Timer> = Arc::new(Default::default());

    let builder: WebviewBuilder<Arc<Timer>> = WebviewBuilder::new();

    let webview: Webview<Arc<Timer>> = builder
        .set_title("Timer")
        .set_content(Content::Html(HTML_DATA))
        .set_width(400)
        .set_height(300)
        .set_userdata(Arc::clone(&timer))
        .set_external_invoke(|webview: &Webview<Arc<Timer>>, arg: &str| {
            let timer = webview.userdata().unwrap();
            match arg {
                "reset" => {
                    timer.set(0);
                    timer.render(webview);
                },
                "exit" => webview.terminate(),
                _ => {},
            };
        })
        .build()
        .expect("...");

    let (main_handle, thread_handle) = webview.thread_handles();
    thread::spawn(move || {
        thread::sleep(time::Duration::from_micros(100_000));
        timer.incr();
        thread_handle.dispatch(|webview: &Webview<Arc<Timer>>| {
            let timer = webview.userdata().unwrap();
            timer.render(webview);
        });
    });

    main_handle.run();
}
