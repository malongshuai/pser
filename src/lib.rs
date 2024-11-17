use lazy_static::lazy_static;
use std::path::PathBuf;

pub use crypt::EncryptData;

pub mod db_file;
pub mod error;
pub mod gen_rand;
pub mod pser;
pub mod verify_header;

lazy_static! {
    pub static ref PSER_MAIN_PASSWD: String = std::env::var("PSER_MAIN_PASSWD").unwrap_or_default();
    pub static ref DB_FILE_HOME: PathBuf = {
        let dir = dirs::home_dir()
            .unwrap()
            .join(".local")
            .join("share")
            .join(".sper");
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("sper.db")
    };
    pub static ref DB_FILE_CUR: PathBuf = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
        .join("pser.db");
}
