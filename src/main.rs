use std::env;

mod app;

fn main() {
    if env::consts::OS != "linux" {
        panic!("We don't support {} yet!", env::consts::OS);
    }
    app::mainloop();
}
