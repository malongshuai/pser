//! 加密数据库的验证头

use crate::error::PserResult;
use chrono_ext::{now8, DateTimeExt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::cell::RefCell;

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyHeader {
    /// 当前分钟的起始Epoch(秒级的Epoch)，即第0秒，
    /// 可通过该字段和try_count来设置每分钟内最大允许解密次数，避免被爆破
    timestamp: RefCell<i64>,
    /// 每次尝试解密，都更新该字段，
    /// 在一分钟内已尝试解密的次数
    try_count: RefCell<u16>,
    /// 主密码对应的Sha512值，用于验证密码的正确性
    verify_data: Vec<u8>,
}

impl VerifyHeader {
    const MAX_TRY: u16 = 200u16;

    pub fn new(main_passwd: &str) -> Self {
        Self {
            timestamp: RefCell::new(cur_min_timestamp()),
            try_count: RefCell::new(0),
            verify_data: passwd_sha512(main_passwd),
        }
    }

    /// 验证给定明文密码是否正确(是否能解开加密头)，还验证是否超出单位时间内的验证限制次数
    pub fn verify_header(&self, main_passwd: &str) -> bool {
        // 返回true表示需要等待，返回false表示可以继续验证
        let try_wait = self.update_header_status();
        !try_wait && self.verify_data == passwd_sha512(main_passwd)
    }

    /// 更新验证头的状态信息，同时返回是否超过单位时间内验证头的验证限制次数，
    /// 超出限制次数返回true，没有超出限制次数返回false
    fn update_header_status(&self) -> bool {
        let cur_m = cur_min_timestamp();
        let mut try_cnt = self.try_count.borrow_mut();
        let t = *self.timestamp.borrow();
        match t == cur_m {
            true => {
                *try_cnt += 1;
            }
            false => {
                *self.timestamp.borrow_mut() = cur_m;
                *try_cnt = 0;
            }
        }

        try_cnt.ge(&Self::MAX_TRY)
    }

    /// bincode序列化的验证头
    pub fn encode(&self) -> Vec<u8> {
        bincode::serialize(self).expect("can't serialize VerifyHeader")
    }

    pub fn decode(data: &[u8]) -> PserResult<Self> {
        Ok(bincode::deserialize::<Self>(data)?)
    }
}

fn cur_min_timestamp() -> i64 {
    now8().zero_from_sec().timestamp()
}

/// 明文密码转换为sha512
fn passwd_sha512(plain_passwd: &str) -> Vec<u8> {
    let mut hasher = Sha512::new();
    hasher.update(plain_passwd.as_bytes());
    let res = hasher.finalize();
    res.to_vec()
}
