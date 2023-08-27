use clap::Parser;
use db_file::PserDB;
use gen_rand::gen_passwd;
use lazy_static::lazy_static;
use opts::{
    ExportCmd, GenPasswdCmd, ImportCmd, ImportSrcType, InsertCmd, QueryCmd, ResetCmd, RmCmd,
};
use pser::Pser;
use std::{collections::HashMap, io::Read, path::PathBuf};

pub use crypt::EncryptData;

pub mod db_file;
pub mod error;
pub mod gen_rand;
pub mod opts;
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

fn main() {
    // add_test_psers();
    // std::process::exit(0);

    let opts = opts::Opts::parse();
    match opts.cmds {
        opts::Cmds::Init => init(),
        opts::Cmds::Query(opt) => query(&opt),
        opts::Cmds::Insert(opt) => add_passwd(&opt),
        opts::Cmds::Rm(opt) => remove_passwd(&opt),
        opts::Cmds::Reset(opt) => reset_main_passwd(&opt),
        opts::Cmds::Gen(opt) => {
            let passwds = gen(&opt);
            if !passwds.is_empty() {
                println!("{}", passwds.join("\n"));
            }
        }
        opts::Cmds::Import(opt) => import(&opt),
        opts::Cmds::Export(opt) => export(&opt),
    }
}

/// 将尝试先读取 PSER_PASSWD 环境变量，如果没有设置该环境变量，将交互式提示输入密码
fn prompt_password(prompt_msg: &str) -> String {
    match PSER_MAIN_PASSWD.is_empty() {
        true => {
            let p = rpassword::prompt_password(format!("{}: ", prompt_msg)).unwrap();
            println!();
            p
        }
        false => PSER_MAIN_PASSWD.clone(),
    }
}

/// 检查密码库是否存在
fn pser_lib_exists() -> bool {
    DB_FILE_HOME.exists() || DB_FILE_CUR.exists()
}

fn init() {
    let main_passwd = prompt_password("输入主密码");
    PserDB::new(&main_passwd).unwrap();
}

fn gen(opt: &GenPasswdCmd) -> Vec<String> {
    let (up, down, num, punc) = (opt.up, opt.down, opt.num, opt.punc);

    // 生成的密码长度
    let len = opt.len as usize;
    (0..opt.cnt)
        .map(|_| gen_passwd(up, down, num, punc, len))
        .filter(|x| !x.is_empty())
        .collect::<Vec<String>>()
}

fn reset_main_passwd(opt: &ResetCmd) {
    if !pser_lib_exists() {
        println!("密码库不存在");
        return;
    }

    let main_passwd = prompt_password("输入旧的主密码");
    let mut db = PserDB::new(&main_passwd).unwrap();
    db.change_passwd(&opt.new_passwd).unwrap();
}

fn remove_passwd(opt: &RmCmd) {
    if !pser_lib_exists() {
        println!("密码库不存在");
        return;
    }

    let main_passwd = prompt_password("输入主密码");

    // 如果是remove，则删除密码库，且只有密码验证通过时才删除密码库文件，验证失败不删除
    if opt.uuid.eq_ignore_ascii_case("remove") {
        let t = PserDB::new(&main_passwd).is_ok();
        if t {
            let _ = std::fs::remove_file(&*DB_FILE_HOME);
            let _ = std::fs::remove_file(&*DB_FILE_CUR);
        }

        return;
    }

    let mut db = PserDB::new(&main_passwd).unwrap();
    // 如果是all，则清空所有密码信息
    if opt.uuid.eq_ignore_ascii_case("all") {
        db.clear().unwrap();
        return;
    }

    for uuid_prefix in opt.uuid.split(",") {
        let uuids = db.uuid_by_prefix(uuid_prefix);
        match uuids.len() {
            1 => db.remove(&uuids[0]).unwrap(),
            0 => println!("Uuid({})不存在", uuid_prefix),
            _ => println!("Uuid({})指定位数过少产生歧义", uuid_prefix),
        }
    }
}

fn query(opt: &QueryCmd) {
    if !pser_lib_exists() {
        println!("密码库不存在");
        return;
    }
    let main_passwd = prompt_password("输入主密码");
    let db = PserDB::new(&main_passwd).unwrap();

    // 为None，表示列出密码库中所有信息，而不是搜索
    let psers = match &opt.str {
        Some(str) => db.query(str),
        None => db.all_pser(),
    };

    let iter = psers.into_iter();
    let mut s: Vec<String> = match opt.short {
        true => iter
            .map(|(uuid, pser)| pser.simple_display(Some(uuid)))
            .collect(),
        false => iter
            .map(|(uuid, pser)| pser.verical_display(Some(uuid)))
            .collect(),
    };
    s.sort();
    println!("{}", s.join("\n"));
}

fn add_passwd(opt: &InsertCmd) {
    if !pser_lib_exists() {
        println!("密码库不存在");
        return;
    }
    let main_passwd = prompt_password("输主密码");
    let mut db = PserDB::new(&main_passwd).unwrap();

    // 更新pser而不是添加pser
    if let Some(uuid_prefix) = &opt.uuid {
        let uuids = db.uuid_by_prefix(uuid_prefix);
        match uuids.len() {
            // 搜索到了要更新的pser
            1 => {
                let mut pser = db.get_pser(&uuids[0]).unwrap().clone();
                update_pser(&mut pser, opt);
                db.update(&uuids[0], pser).unwrap();
            }
            0 => println!("指定的Uuid({})不存在", uuid_prefix),
            _ => println!("Uuid({})指定位数过少产生歧义", uuid_prefix),
        }
        return;
    }

    // 添加新的pser
    let mut pser = Pser::new();
    update_pser(&mut pser, opt);
    db.insert(pser).unwrap();
}

/// 根据给定的AddCmd中的选项，更新给定的pser
fn update_pser(pser: &mut Pser, opt: &InsertCmd) {
    if let Some(user) = &opt.user {
        pser.set_username(user);
    }

    if let Some(url) = &opt.url {
        pser.set_url(url);
    }

    if let Some(desc) = &opt.desc {
        pser.set_desc(desc);
    }

    // 如果首字符是`+`，则追加，否则覆盖
    if let Some(email) = &opt.email {
        match email.strip_prefix('+') {
            Some(email) => {
                pser.email.push(',');
                pser.email.push_str(email);
            }
            None => {
                pser.set_email(&email);
            }
        }
    }

    // 如果首字符是`+`，则追加，否则覆盖
    if let Some(phone) = &opt.phone {
        match phone.strip_prefix('+') {
            Some(phone) => {
                pser.phone.push(',');
                pser.phone.push_str(phone);
            }
            None => {
                pser.set_phone(&phone);
            }
        }
    }

    if let Some(passwd) = &opt.passwd {
        pser.set_passwd(&passwd);
    }

    // 如果首字符是`+`，则追加，否则覆盖
    if let Some(comment) = &opt.comment {
        // let comment = comment.split([',', ';']).collect::<Vec<_>>().join("\n");
        match comment.strip_prefix('+') {
            Some(comment) => {
                pser.phone.push(',');
                pser.comment.push_str(comment);
            }
            None => {
                pser.set_comment(comment);
            }
        }
    }
}

fn import(opt: &ImportCmd) {
    if !pser_lib_exists() {
        println!("密码库不存在");
        return;
    }
    let main_passwd = prompt_password("输主密码");
    let mut db = PserDB::new(&main_passwd).unwrap();

    // 读取等待导入的数据
    let input_str = match &opt.input {
        Some(f) => std::fs::read_to_string(f).unwrap(),
        // 从标准输入中读取等待导入的数据
        None => {
            let mut stdin = std::io::stdin();
            let mut buf = String::new();
            stdin.read_to_string(&mut buf).unwrap();
            buf
        }
    };

    match opt.src_type {
        // json的数据，来自本程序自身的导出，因此直接导入到当前数据库
        ImportSrcType::Json => import_from_json(&mut db, &input_str),
        ImportSrcType::Csv => import_from_csv(&mut db, &input_str),
    };
}

fn import_from_json(db: &mut PserDB, json_str: &str) {
    let s: HashMap<String, Pser> =
        serde_json::from_str(&json_str).expect(&format!("can't decode: {}", json_str));

    let mut success_insert = 0;

    for (uuid, pser) in s {
        let uuids = db.uuid_by_prefix(&uuid);
        match uuids.is_empty() {
            // 如果当前库中已经存在重复的uuid，则重新生成新的uuid
            false => {
                println!("Uuid({})已经存在，可能和库中的密码重复，但仍将生成新的uuid并插入，如果新导入的密码确实存在重复，可稍后手动删除", uuid);
                db.insert(pser).unwrap();
            }
            true => {
                db.insert(pser).unwrap();
            }
        }
        success_insert += 1;
    }

    println!("成功插入 {} 条密码信息", success_insert);
}

fn import_from_csv(db: &mut PserDB, csv_str: &str) {
    let mut rdr = csv::Reader::from_reader(csv_str.as_bytes());

    // csv文件中的name username url password comment字段可能是乱的，因此先找出各字段在每行记录上的索引
    let header = rdr.headers().expect("missing csv header").into_iter();
    let name_idx = header
        .clone()
        .position(|x| x == "name")
        .unwrap_or(usize::MAX);
    let comment_idx = header
        .clone()
        .position(|x| x == "comment")
        .unwrap_or(usize::MAX);
    let username_idx = header
        .clone()
        .position(|x| x == "username")
        .expect("missing username field in csv");
    let url_idx = header
        .clone()
        .position(|x| x == "url")
        .expect("missing url field in csv");
    let passwd_idx = header
        .clone()
        .position(|x| x == "password")
        .expect("missing password field in csv");

    let mut line_num = 1;
    for result in rdr.records() {
        line_num += 1;
        let res = result.unwrap();

        let desc = res.get(name_idx);
        let comment = res.get(comment_idx);
        let username = res
            .get(username_idx)
            .expect(&format!("missing username field at line {}", line_num));
        let url = res
            .get(url_idx)
            .expect(&format!("missing url field at line {}", line_num));
        let passwd = res
            .get(passwd_idx)
            .expect(&format!("missing password field at line {}", line_num));

        let mut pser = Pser::new();
        pser.set_username(username).set_url(url).set_passwd(passwd);
        if let Some(desc) = desc {
            pser.set_desc(desc);
        }
        if let Some(comment) = comment {
            pser.set_comment(comment);
        }

        db.insert(pser).unwrap();
    }
    println!("成功插入 {} 条密码信息", line_num - 1);
}

fn export(opt: &ExportCmd) {
    if !pser_lib_exists() {
        println!("密码库不存在");
        return;
    }
    let main_passwd = prompt_password("输入主密码");
    let db = PserDB::new(&main_passwd).unwrap();

    let psers: HashMap<&String, &Pser> = db.all_pser().into_iter().collect();
    let str = serde_json::to_string_pretty(&psers).unwrap();

    match &opt.output {
        Some(f) => {
            std::fs::write(f, str).unwrap();
        }
        None => println!("{}", str),
    }
}

#[allow(dead_code)]
fn add_test_psers() {
    let mut db = PserDB::new("helloworld").unwrap();

    let mut pser1 = Pser::new();
    pser1
        .set_username("jma")
        .set_url("google.com")
        .set_desc("google")
        .set_email("jma@hotmail.com")
        .set_phone("18213812341")
        .set_passwd("passwd");
    db.insert(pser1).unwrap();

    let mut pser2 = Pser::new();
    pser2
        .set_username("kex")
        .set_url("outlook.com")
        .set_email("kex@163.com")
        .set_passwd("passwd");
    db.insert(pser2).unwrap();
}

#[cfg(test)]
mod t {
    #[test]
    fn tt() {
        let str = "skdflajs,d";
        println!("{:?}", str.split(",").collect::<Vec<_>>());
    }
}
