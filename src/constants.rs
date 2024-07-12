use dotenvy::dotenv;
use lazy_static::lazy_static;

fn set_bool_mode(param: &str) -> bool {
    dotenv().ok();
    let mode = std::env::var(param).unwrap_or("false".to_owned());
    mode.parse::<bool>().unwrap()
}

pub mod env {
    pub const DEBUG_MODE_ENV_VAR: &str = "DEBUG_MODE";
    pub const REMOVE_CONN_ENV_VAR: &str = "REMOVE_CONN";

}

lazy_static! {
    pub static ref DEBUG_MODE: bool = set_bool_mode(env::DEBUG_MODE_ENV_VAR);
    pub static ref REMOVE_CONN: bool = set_bool_mode(env::REMOVE_CONN_ENV_VAR);
}
