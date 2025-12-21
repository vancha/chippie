use std::env;

fn main() {
    let wayland = env::var_os("CARGO_FEATURE_WAYLAND").is_some();
    let x11 = env::var_os("CARGO_FEATURE_X11").is_some();

    if !wayland && !x11 {
        panic!("You must enable at least one of the features: wayland or x11");
    }
}
