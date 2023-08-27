use chrono_ext::{east8, now8, EpochToDateTimeExt};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Write};

/// ```
/// // 按需调用`set_xxx()`方法
/// let mut pser = Pser::new();
/// pser.set_username("juji")
///     .set_url("google.com")
///     .set_desc("google")
///     .set_passwd("juji@ha124")
///     .set_email("juji@hotmail.com")
///     .set_phone("18812345678")
///     .set_comment("card_num:9120837490102991");
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pser {
    /// 账户名/用户名
    pub username: String,

    /// 哪个网站的账户(基于域名)
    pub url: String,

    /// 什么账户，可以结合url或者和url二选一
    /// 例如google的账户，可以记录为`url:google.com`或者`desc:google`，或者两者都记录
    pub desc: String,

    /// 邮箱，如果有第二邮箱，则应该使用备注保存
    pub email: String,

    /// 联系方式，如果有第二联系方式，则应该使用备注保存
    pub phone: String,

    /// 密码，可以是单个密码，也可以是逗号分隔的多个单词助记词
    pub passwd: String,

    /// 备注，记录额外信息
    pub comment: String,

    /// 历史密码信息。密码被修改后，旧密码保存在此  
    ///
    /// - key: 被修改的时间点(秒级Epoch)
    ///
    /// - value: 被修改的旧密码(可能是纯密码，可能是多个逗号分隔的单词助记词)
    pub history: HashMap<i64, String>,
}

impl Pser {
    pub fn new() -> Self {
        Self::default()
    }

    /// 给定字符串，查询是否是该账号。例如，查询是否是google的账号
    ///
    /// 只查询 url 和 desc 两个字段
    pub fn is_me(&self, str: &str) -> bool {
        let str = str.to_lowercase();
        self.url.to_ascii_lowercase().contains(&str)
            || self.desc.to_ascii_lowercase().contains(&str)
    }

    pub fn set_username(&mut self, username: &str) -> &mut Self {
        self.username = username.to_string();
        self
    }

    pub fn set_url(&mut self, url: &str) -> &mut Self {
        let domain = domain_from_url(url).expect("unsupport url format");
        self.url = domain.to_string();
        self
    }

    pub fn set_desc(&mut self, desc: &str) -> &mut Self {
        self.desc = desc.to_string();
        self
    }

    pub fn set_email(&mut self, email: &str) -> &mut Self {
        self.email = email.to_string();
        self
    }

    pub fn set_phone(&mut self, phone: &str) -> &mut Self {
        self.phone = phone.to_string();
        self
    }

    pub fn set_passwd(&mut self, passwd: &str) -> &mut Self {
        let old_passwd = std::mem::replace(&mut self.passwd, passwd.to_string());
        if !old_passwd.is_empty() {
            self.history.insert(now8().timestamp(), old_passwd);
        }
        self
    }

    pub fn set_comment(&mut self, comment: &str) -> &mut Self {
        self.comment = comment.to_string();
        self
    }

    /// 查看所有被修改过的旧密码
    /// Vec<(被修改时间点，被修改的旧密码)>
    pub fn history_passwds(&self) -> Vec<(String, String)> {
        self.history
            .iter()
            .map(|(k, v)| (k.secs_to_dt(east8()).to_string(), v.to_owned()))
            .collect::<Vec<(String, String)>>()
    }

    pub fn simple_display(&self, uuid: Option<&str>) -> String {
        let mut str = String::new();

        if !self.url.is_empty() {
            let _ = write!(&mut str, "url:{}", self.url);
        }
        if !self.username.is_empty() {
            let _ = write!(&mut str, "|账户名:{}", self.username);
        }
        if !self.passwd.is_empty() {
            let _ = write!(&mut str, "|密码:{}", self.passwd);
        }
        if let Some(uuid) = uuid {
            let _ = write!(&mut str, "|UUID:{}", uuid);
        }

        str
    }

    pub fn verical_display(&self, uuid: Option<&str>) -> String {
        let mut str = String::new();

        if let Some(uuid) = uuid {
            let _ = writeln!(&mut str, "-[ UUID: {} ]----------------------", uuid);
        }

        if !self.username.is_empty() {
            let _ = writeln!(&mut str, "账户名(account): {}", self.username);
        }
        if !self.desc.is_empty() {
            let _ = writeln!(&mut str, "所属(desc): {}", self.desc);
        }
        if !self.url.is_empty() {
            let _ = writeln!(&mut str, "所属网站(url): {}", self.url);
        }

        if !self.email.is_empty() {
            let _ = writeln!(&mut str, "邮箱(email): {}", self.email);
        }

        if !self.phone.is_empty() {
            let _ = writeln!(&mut str, "联系方式(phone): {}", self.phone);
        }

        if !self.passwd.is_empty() {
            let _ = writeln!(&mut str, "密码(passwd): {}", self.passwd);
        }

        if !self.comment.is_empty() {
            let comment = self.comment.split([',', ';']).collect::<Vec<_>>();
            let _ = writeln!(&mut str, "备注(comment): {}", comment.join("\n"));
            // let _ = writeln!(&mut str, "备注(comment): {}", self.comment);
        }

        str
    }
}

/// key： Uuid(Simple)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Psers(HashMap<String, Pser>);

impl Psers {
    pub fn inner(&self) -> &HashMap<String, Pser> {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut HashMap<String, Pser> {
        &mut self.0
    }
}

/// 从URL中获取域名部分，
///
/// 从`schema://_x.x_/y_`中获取`_x.x_`，即协议之后(协议可省略)，Path之前的内容(Path可省略)
///
/// 例如，"http://id1.cloud.abc.com/a/b/c.html"，将得到`id1.cloud.abc.com`
fn domain_from_url(url: &str) -> Option<&str> {
    let re = Regex::new(r"^(?:.*://)?(?<domain>.*?)(?:/|$)").unwrap();
    let capt = re.captures(url).unwrap();
    capt.name("domain").and_then(|x| Some(x.as_str()))
}

#[cfg(test)]
mod t {
    use regex::Regex;

    #[test]
    fn tt() {
        let re = Regex::new(r"^(?:.*://)?(?<domain>.*?)/").unwrap();
        let url = "http://id1.cloud.abc.com/a/b/c.html";
        let capt = re.captures(url).unwrap();
        println!("{:?}", capt.name("domain").and_then(|x| Some(x.as_str())));
    }
}
