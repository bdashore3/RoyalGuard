use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufReader;

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    pub bot_token: String,
    pub application_id: u64,
    pub default_prefix: String,
    pub db_connection: String,
}

pub fn read_creds(path: String) -> Result<Credentials, Box<dyn std::error::Error + 'static>> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);

    let info: Credentials = serde_json::from_reader(reader).unwrap();

    Ok(info)
}
