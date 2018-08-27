extern crate webview_rs;

use std::sync::{Arc, Mutex};
use std::time;
use std::thread;

use webview_rs::{Webview, WebviewHandle, Builder, Content};

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

    fn render(&self, webview: &mut Webview<Arc<Timer>>) {
        let ticks = self.get();
        webview.eval_fn("updateTicks", &["ticks"]);
    }
}

fn main() {
    let timer: Arc<Timer> = Arc::new(Default::default());

    let builder: Builder<Arc<Timer>> = Builder::new();

    let mut webview: WebviewHandle<Arc<Timer>> = Builder::new()
        .set_title("Timer")
        .set_content(Content::Html(HTML_DATA))
        .set_size(400, 400)
        .set_userdata(Arc::clone(&timer))
        .set_external_invoke(|webview: &mut Webview<Arc<Timer>>, arg: &str| {
            match arg {
                "reset" => {
                    {
                        let timer = webview.userdata().unwrap();
                        timer.set(0);
                    }
                    webview.eval_fn("updateTicks", &["0"]).unwrap();
                },
                "exit" => webview.terminate(),
                _ => {},
            };
        })
        .build()
        .expect("...");

    let thread_handle = webview.thread_handle();

    thread::spawn(move || {
        loop {
            thread::sleep(time::Duration::from_micros(100_000));
            timer.incr();
            let result = thread_handle.try_dispatch(|webview| {
                let ticks = {
                    let timer = webview.userdata().unwrap();
                    timer.get()
                };
                webview.eval_fn("updateTicks", &[&ticks.to_string()]).unwrap();
            });

            if result.is_err() {
                break;
            }
        }
    });

    webview.run(true);
}
