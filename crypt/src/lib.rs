use anyhow::{anyhow, Ok};
use chacha20poly1305::{
    aead::{rand_core::RngCore, Aead, OsRng},
    ChaCha20Poly1305, KeyInit,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EncryptData {
    /// salt
    s: Vec<u8>,
    /// nonce
    n: Vec<u8>,
    /// data
    d: Vec<u8>,
}

impl EncryptData {
    /// 加密，并使用bincode对加密后的数据进行序列化
    pub fn encrypt<T>(data: &T, passwd: &str) -> Result<Vec<u8>, anyhow::Error>
    where
        T: Serialize,
    {
        let v8 = bincode::serialize(data)?;
        let encrypt_data = Self::inner_encrypt(v8, passwd)?;
        let encrypt_data = bincode::serialize(&encrypt_data).unwrap();
        Ok(encrypt_data)
    }

    /// 对加密后的数据(bincode序列化之后的加密数据)进行解密
    pub fn decrypt<S>(enc_data: &[u8], passwd: &str) -> Result<S, anyhow::Error>
    where
        S: DeserializeOwned,
    {
        // let bincode_data = hex::decode(enc_data)?;
        let encrypt_data = bincode::deserialize::<Self>(enc_data)?;
        let plain_data_vec = encrypt_data.inner_decrypt(passwd)?;
        bincode::deserialize::<S>(&plain_data_vec)
            .map_err(|e| anyhow!("bincode deserialize error: {}", e))
    }

    /// 生成96bit(12bytes)的nonce
    fn gen_nonce() -> [u8; 12] {
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }

    /// 生成salt，长度至少8位
    fn gen_salt() -> [u8; 8] {
        let mut salt = [0u8; 8];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    /// 给定明文指定的密码，根据 argon2 生成安全的指定长度的密码(hash之后的)
    fn gen_passwd(passwd: &str, salt: &[u8]) -> Vec<u8> {
        let config = argon2::Config {
            hash_length: 32,
            ..argon2::Config::default()
        };
        argon2::hash_raw(passwd.as_bytes(), salt, &config).unwrap()
    }

    fn inner_encrypt(data: Vec<u8>, passwd: &str) -> Result<Self, anyhow::Error> {
        let salt = Self::gen_salt();
        let passwd_key = Self::gen_passwd(passwd, &salt);

        let key = chacha20poly1305::Key::from_slice(&passwd_key);
        let cipher = ChaCha20Poly1305::new(key);

        let nonce = Self::gen_nonce();
        let nonce = chacha20poly1305::Nonce::from_slice(&nonce);

        let cipher_ctx = cipher
            .encrypt(nonce, data.as_ref())
            .map_err(|e| anyhow!("encrypt error: {}", e))?;
        let encrypt_data = EncryptData {
            s: salt.to_vec(),
            n: nonce.to_vec(),
            d: cipher_ctx,
        };
        Ok(encrypt_data)
    }

    fn inner_decrypt(&self, passwd: &str) -> Result<Vec<u8>, anyhow::Error> {
        let passwd_key = Self::gen_passwd(passwd, &self.s);
        let key = chacha20poly1305::Key::from_slice(&passwd_key);
        let cipher = ChaCha20Poly1305::new(key);

        let nonce = &self.n;
        let nonce = chacha20poly1305::Nonce::from_slice(nonce);
        cipher
            .decrypt(nonce, self.d.as_ref())
            .map_err(|e| anyhow!("decrypt error: {}", e))
    }
}

#[cfg(test)]
mod d {
    use serde::{Deserialize, Serialize};

    use crate::EncryptData;
    #[derive(Debug, Serialize, Deserialize)]
    struct S {
        key: String,
        user: String,
        ip: String,
    }

    #[test]
    fn m() {
        let data = S {
            key: "fkdlasd".into(),
            user: "longshuai".into(),
            ip: "192.168.100.111".into(),
        };

        let passwd = "thisiskey";
        let enc_data = EncryptData::encrypt(&data, passwd).unwrap();
        let ss = EncryptData::decrypt::<S>(&enc_data, passwd);
        println!("enc_str: {:?}", enc_data);
        println!("ss: {:?}", ss);
    }
}
