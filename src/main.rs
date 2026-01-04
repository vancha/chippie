use std::{
    cell::RefCell,
    fs::{self, OpenOptions},
    io::Write,
    rc::Rc,
};

use chippie_gui::Application;

const CONFIGURATION_FILE: &str = "Configuration.toml";

fn main() {
    let settings = Rc::new(RefCell::new(
        fs::read_to_string(CONFIGURATION_FILE)
            .ok()
            .and_then(|data| toml::from_str(&data).unwrap())
            .unwrap_or_default(),
    ));
    let _ = Application::run(settings.clone());

    let data = toml::to_string(&(Rc::into_inner(settings).unwrap()).into_inner()).unwrap();
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(CONFIGURATION_FILE)
        .unwrap();
    file.write_all(data.as_bytes()).unwrap();
}
