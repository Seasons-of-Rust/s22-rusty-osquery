use std::env;

mod app;

fn main() {
    if env::consts::OS != "linux" {
        panic!("[CRITICAL HIT] Oh Nos! We don't support {} yet!", env::consts::OS);
    }
    app::mainloop();
}
