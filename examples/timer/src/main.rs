use std::sync::{Arc, Mutex};
use std::time;
use std::thread;

static HTML_DATA: &'static str = "";

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

    fn render(&self, webview: &Webview) {
        let ticks = self.get();
        webview.eval(&format!("updateTicks({})", ticks));
    }
}

fn main() {
    let timer: Arc<Timer> = Arc::new(Default::default());

    let webview = WebviewBuilder::new()
        .set_title("Timer")
        .set_content(Content::Html(HTML_DATA))
        .set_width(400)
        .set_height(300)
        .set_userdata(Arc::clone(&timer)) //owning vs borrowing
        .set_external_invoke(|webview: &Webview, arg: &str| {
            let timer: &Arc<Timer> = &webview.userdata();
            match arg {
                "reset" => {
                    timer.set(0);
                    timer.render(webview);
                },
                "exit" => webview.terminate()
            };
        })
        .build()
        .expect("...");

    let (main_handle, thread_handle) = webview.dispatch_handles();

    main_handle.run(move || {
        thread::spawn(move || {
            loop {
                thread::sleep(time::Duration::from_micros(100_000));
                timer.incr();
                thread_handle.dispatch(|webview: &Webview| { //Does not need args, can use closure/capture
                    webview.userdata().render(webview);
                });
            }
        });
    });
}
