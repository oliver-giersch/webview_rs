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
        webview.eval_fn("updateTicks", &["ticks"]).unwrap();
    }
}

type Userdata = Arc<Timer>;

fn external_invoke(webview: &mut Webview<Arc<Timer>>, arg: &str) {
    match arg {
        "reset" => reset_invoke(webview),
        "exit" => exit_invoke(webview),
        _ => {},
    }
}

fn reset_invoke(webview: &mut Webview<Userdata>) {
    {
        webview.userdata().set(0);
    }
    webview.eval_fn("updateTicks", &["0"]).unwrap();
}

fn exit_invoke(webview: &mut Webview<Userdata>) {
    webview.terminate();
}

fn main() {
    let timer: Arc<Timer> = Arc::new(Default::default());
    let mut webview = Builder::with_userdata(Arc::clone(&timer))
        .set_title("Timer")
        .set_content(Content::Html(HTML_DATA))
        .set_size(400, 400)
        .set_external_invoke(external_invoke) //|webview: &mut Webview<Arc<Timer>>, arg: &str| {...}
        .build()
        .expect("...");

    let thread_handle = webview.thread_handle();

    thread::spawn(move || {
        loop {
            thread::sleep(time::Duration::from_millis(100));
            timer.incr();
            let result = thread_handle.try_dispatch(|webview| {
                let ticks = webview.userdata().get();
                webview.eval_fn("updateTicks", &[&ticks.to_string()]).unwrap();
            });

            if result.is_err() {
                break;
            }
        }
    });

    webview.run(true);
}
