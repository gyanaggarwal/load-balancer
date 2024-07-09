use dotenvy::dotenv;
use lazy_static::lazy_static;

fn set_debug_mode() -> bool {
    dotenv().ok();
    let mode = std::env::var(env::DEBUG_MODE_ENV_VAR).unwrap_or("false".to_owned());
    mode.parse::<bool>().unwrap()
}

pub mod env {
    pub const DEBUG_MODE_ENV_VAR: &str = "DEBUG_MODE";
}

lazy_static! {
    pub static ref DEBUG_MODE: bool = set_debug_mode();
}
