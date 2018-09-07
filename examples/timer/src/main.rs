extern crate webview_rs;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use webview_rs::{Arg, Builder, Content, Webview};

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
}

type Userdata = Arc<Timer>;

//Could use a closure instead
fn external_invoke(webview: &mut Webview, userdata: &mut Userdata, arg: &str) {
    match arg {
        "reset" => reset_invoke(webview, userdata),
        "exit" => exit_invoke(webview),
        _ => {}
    }
}

fn reset_invoke(webview: &mut Webview, userdata: &mut Userdata) {
    userdata.set(0);
    webview.eval_fn("updateTicks", &[Arg::Int(0)]).unwrap();
}

fn exit_invoke(webview: &mut Webview) {
    webview.terminate();
}

fn main() {
    let timer: Arc<Timer> = Arc::new(Default::default());
    let mut webview = Builder::with_userdata(Arc::clone(&timer))
        .set_title("Timer")
        .set_content(Content::Html(HTML_DATA))
        .set_size(400, 400)
        .set_external_invoke(external_invoke)
        //.set_external_invoke(|webview, userdata, arg| { ... })
        .build()
        .expect("could not create webview");

    let thread_handle = webview.thread_handle();

    thread::spawn(move || loop {
        thread::sleep(time::Duration::from_millis(100));
        timer.incr();
        let result = thread_handle.try_dispatch(|webview, userdata| {
            let ticks = userdata.get();
            webview.eval_fn("updateTicks", &[Arg::Int(ticks)]).unwrap();
        });

        if result.is_err() {
            break;
        }
    });

    webview.run(true);
}
