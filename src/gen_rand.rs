use rand::Rng;

const AZ_UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const AZ_LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const NUM: &[u8] = b"0123456789";
const PUNC: &[u8] = b")(*&^%$#@!~";

/// 生成随机密码
///
/// up: 生成的密码是否包含大写字母字符集`A-Z`
/// 
/// down: 生成的密码是否包含小写字母字符集`a-z`
/// 
/// num: 生成的密码是否包含数字字符集`0-9`
/// 
/// punc: 生成的密码是否包含一些特殊符号字符集`)(*&^%$#@!~`
/// 
/// len: 生成的密码长度
///
/// 注意：即使指定了密码要包含哪类字符集，生成的随机密码中也不一定会包含这类字符，
/// 某类设置为true，仅表示这类字符集会纳入考虑
pub fn gen_passwd(up: bool, down: bool, num: bool, punc: bool, len: usize) -> String {
    if len == 0 {
        return String::new();
    }

    let mut chars: Vec<u8> = vec![];
    if up {
        chars.extend(AZ_UPPER);
    }

    if down {
        chars.extend(AZ_LOWER);
    }

    if num {
        chars.extend(NUM);
    }

    if punc {
        chars.extend(PUNC);
    }

    let chars_len = chars.len();
    if chars_len == 0 {
        return String::new();
    }

    let mut rng = rand::thread_rng();
    let password: String = (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..chars_len);
            chars[idx] as char
        })
        .collect();

    password
}

#[cfg(test)]
mod tt {
    use crate::gen_rand::gen_passwd;

    #[test]
    fn t() {
        println!("{}", gen_passwd(false, true, true, true, 8));
    }
}
