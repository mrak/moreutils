use std::env;

pub fn get_editor() -> String {
    env::var("VISUAL")
        .map_err(|_| env::var("EDITOR"))
        .unwrap_or("vi".to_owned())
}
