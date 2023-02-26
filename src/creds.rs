// Definitions for credentials

use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::io::Error;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Credential {
    pub name: String,
    pub account: String,
    encrypted_password: String,
}

impl Credential {
    pub fn decrypt(&self, key: &str) -> Result<String, Error> {
        let mc = new_magic_crypt!(key, 256);
        Ok(mc
            .decrypt_base64_to_string(&self.encrypted_password)
            .unwrap())
    }

    pub fn from_input(
        name: &str,
        username: &str,
        password: &str,
        key: &str,
    ) -> Result<Credential, Error> {
        let mc = new_magic_crypt!(key, 256);
        Ok(Credential {
            name: name.to_string(),
            account: username.to_string(),
            encrypted_password: mc.encrypt_str_to_base64(&password),
        })
    }
    pub fn as_json(&self) -> Result<String, Error> {
        Ok(json!(&self).to_string())
    }
    pub fn from_file(src: &str) -> Result<Credential, Error> {
        let data = std::fs::read_to_string(src)?;
        Ok(serde_json::from_str::<Credential>(&data).unwrap())
    }
}
