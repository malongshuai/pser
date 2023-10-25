//! 处理加密数据库文件
//!
//! 格式：验证头大小(u16) + 验证头 + 加密数据
//!   验证头大小：VerifyHeader Type 的 bincode 序列化后的长度
//!   验证头：VerifyHeader Type 的 bincode 序列化
//!   加密数据：密码数据被加密后的 EncryptData 的 bincode 格式

use crate::{
    error::{Error, PserResult},
    pser::{Pser, Psers},
    verify_header::VerifyHeader,
    DB_FILE_CUR, DB_FILE_HOME,
};
use crypt::EncryptData;
use redb::{Database, ReadableTable, TableDefinition};
use sha2::{Digest, Sha512};
use std::{io, path::Path};
use uuid::Uuid;

/// 表名(该表的key为&str，value为bincode编码后的字节数据)
const TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("passwd");
/// TABLE表中代表验证头数据的key
const HEADER_KEY: &str = "header";
/// TABLE表中代表数据部分的key
const DATA_KEY: &str = "data";

pub struct SyncDb {
    /// 家目录下的加密数据库`$HOME/.pser/pser.db`
    db_home: Database,
    /// 可执行程序所在目录下的加密数据库`<DIR>/pser.db`
    db_cur: Database,
}

impl SyncDb {
    pub fn new() -> PserResult<Self> {
        let s = match (DB_FILE_HOME.exists(), DB_FILE_CUR.exists()) {
            (true, true) => {
                // 检查主从密码库文件的sha2值，判断主从密码库的数据是否一致
                check_db_sha2();

                std::fs::copy(&*DB_FILE_HOME, &*DB_FILE_CUR)?;
                let db_home = redb::Database::create(&*DB_FILE_HOME).map_err(redb::Error::from)?;
                let db_cur = redb::Database::create(&*DB_FILE_CUR).map_err(redb::Error::from)?;
                Self { db_home, db_cur }
            }
            (true, false) => {
                std::fs::copy(&*DB_FILE_HOME, &*DB_FILE_CUR)?;
                let db_home = redb::Database::create(&*DB_FILE_HOME).map_err(redb::Error::from)?;
                let db_cur = redb::Database::create(&*DB_FILE_CUR).map_err(redb::Error::from)?;
                Self { db_home, db_cur }
            }
            (false, true) => {
                std::fs::copy(&*DB_FILE_CUR, &*DB_FILE_HOME)?;
                let db_cur = redb::Database::create(&*DB_FILE_CUR).map_err(redb::Error::from)?;
                let db_home = redb::Database::create(&*DB_FILE_HOME).map_err(redb::Error::from)?;
                Self { db_home, db_cur }
            }
            // 都不存在，则创建两个，同时创建TABLE表
            (false, false) => {
                let db_home = redb::Database::create(&*DB_FILE_HOME).map_err(redb::Error::from)?;
                let db_cur = redb::Database::create(&*DB_FILE_CUR).map_err(redb::Error::from)?;
                Self::create_table(&db_home).map_err(redb::Error::from)?;
                Self::create_table(&db_cur).map_err(redb::Error::from)?;

                Self { db_home, db_cur }
            }
        };

        Ok(s)
    }

    /// 从Self::TABLE表中读取指定key的数据，返回字节数据(Vec格式)
    pub fn read_db(&self, key: &str) -> Result<Option<Vec<u8>>, redb::Error> {
        let open_trx = self.db_home.begin_read()?;
        let table = open_trx.open_table(TABLE)?;
        let res = table.get(key)?;
        match res {
            Some(res) => Ok(Some(res.value().to_vec())),
            None => Ok(None),
        }
    }

    pub fn write_db(&self, key: &str, data: &[u8]) -> Result<(), redb::Error> {
        Self::_write_db(&self.db_home, key, data)?;
        Self::_write_db(&self.db_cur, key, data)?;
        Ok(())
    }

    /// 表是否空
    pub fn is_empty(&self) -> Result<bool, redb::Error> {
        let read_trx = self.db_home.begin_read()?;
        let tab = read_trx.open_table(TABLE)?;
        Ok(tab.is_empty()?)
    }

    /// 建表，初始化时调用
    fn create_table(db: &Database) -> Result<(), redb::Error> {
        let open_trx = db.begin_write()?;
        {
            let _ = open_trx.open_table(TABLE)?;
        }
        open_trx.commit()?;

        Ok(())
    }

    fn _write_db(db: &Database, key: &str, data: &[u8]) -> Result<(), redb::Error> {
        let open_trx = db.begin_write()?;
        {
            let mut table = open_trx.open_table(TABLE)?;
            table.insert(key, data)?;
        }
        open_trx.commit()?;

        Ok(())
    }
}

pub struct PserDB {
    db: SyncDb,
    /// 主密码：解密整个程序的明文密码
    main_passwd: String,
    /// 验证头(验证主密码是否正确)
    header: VerifyHeader,
    /// 保存或等待保存的各个密码(这些密码通过主密码加密)
    psers: Psers,
}

impl PserDB {
    /// 如果数据库文件存在，则读取库中的验证头和数据，并验证读取的验证头是否正确，
    /// 如果数据库文件不存在，则创建新库写入验证头
    ///
    /// 以下几种情况返回Error：
    /// - 无法打开数据库
    /// - 读验证头失败
    /// - 验证失败(包括超出了单位时间内的验证限制次数以及密码验证失败)
    /// - 读取数据失败
    pub fn new(main_passwd: &str) -> PserResult<Self> {
        let db = SyncDb::new()?;
        if db.is_empty()? {
            let s = Self {
                db,
                main_passwd: main_passwd.to_string(),
                header: VerifyHeader::new(main_passwd),
                psers: Psers::default(),
            };

            s.sync_header()?;
            return Ok(s);
        }

        let header = Self::load_header(&db)?.ok_or(Error::HeaderError)?;
        let mut s = Self {
            db,
            main_passwd: main_passwd.to_string(),
            header,
            psers: Psers::default(),
        };
        // 每次都验证头(包括验证主密码是否正确，以及是否超出验证次数限制)，并将验证更新后的验证头入库
        {
            let verify_flag = s.header.verify_header(main_passwd);
            s.sync_header()?;
            if !verify_flag {
                eprintln!("密码错误");
                std::process::exit(1);
                // return Err(Error::HeaderError);
            }
        }

        let pswd = s.load_psers()?.unwrap_or_default();
        s.psers = pswd;

        Ok(s)
    }

    /// 从数据库中读取验证头
    pub fn load_header(db: &SyncDb) -> PserResult<Option<VerifyHeader>> {
        if let Some(bytes) = db.read_db(HEADER_KEY)? {
            let header = VerifyHeader::decode(&bytes)?;
            return Ok(Some(header));
        }
        Ok(None)
    }

    /// 向数据库中写入验证头
    pub fn sync_header(&self) -> Result<(), redb::Error> {
        let header_bytes = self.header.encode();
        self.db.write_db(HEADER_KEY, &header_bytes)?;
        Ok(())
    }

    /// 从数据库中读取加密数据并解密
    pub fn load_psers(&self) -> PserResult<Option<Psers>> {
        if let Some(bytes) = self.db.read_db(DATA_KEY)? {
            let pser = EncryptData::decrypt::<Psers>(&bytes, &self.main_passwd)?;
            return Ok(Some(pser));
        }
        Ok(None)
    }

    /// 将密码数据进行加密，然后写入数据库
    pub fn sync_psers(&self) -> PserResult<()> {
        let encrypt_data = EncryptData::encrypt(&self.psers, &self.main_passwd)?;
        self.db.write_db(DATA_KEY, &encrypt_data)?;
        Ok(())
    }
}

impl PserDB {
    /// 修改解密程序的明文主密码
    // 除了需要修改并保存验证头，还需要将当前的密码数据用新密码全部重新加密并保存
    pub fn change_passwd(&mut self, plain_passwd: &str) -> PserResult<()> {
        self.header = VerifyHeader::new(plain_passwd);
        self.main_passwd = plain_passwd.to_string();
        self.sync_header()?;
        self.sync_psers()
    }

    /// 添加Pser并保存(将自动生成一个Uuid)
    pub fn insert(&mut self, pser: Pser) -> PserResult<()> {
        self.psers
            .inner_mut()
            .insert(Uuid::new_v4().as_simple().to_string(), pser);
        self.sync_psers()
    }

    /// 替换已存在的Pser并保存(如果uuid不存在，则新创建)
    pub fn update(&mut self, uuid: &str, pser: Pser) -> PserResult<()> {
        self.psers.inner_mut().insert(uuid.to_string(), pser);
        self.sync_psers()
    }

    /// 所有已保存的密码信息
    pub fn all_pser(&self) -> Vec<(&String, &Pser)> {
        self.psers.inner().iter().collect()
    }

    /// 给定域名字符串(例如google.com)或账户所属字符串(例如google)，查询所有账号
    ///
    /// 可能有多个账号
    pub fn query(&self, str: &str) -> Vec<(&String, &Pser)> {
        let mut psers = vec![];
        for (uuid, pser) in self.psers.inner().iter() {
            if pser.is_me(str) {
                psers.push((uuid, pser));
            }
        }

        psers
    }

    /// 根据uuid前缀，搜索uuid key，有可能搜索出多个
    pub fn uuid_by_prefix(&self, uuid_prefix: &str) -> Vec<String> {
        self.psers
            .inner()
            .keys()
            .filter(|x| x.starts_with(uuid_prefix))
            .map(|x| x.to_string())
            .collect()
    }

    /// 根据uuid删除密码库中的密码信息
    pub fn remove(&mut self, uuid: &str) -> PserResult<()> {
        self.psers.inner_mut().remove(uuid);
        self.sync_psers()
    }

    /// 清空密码库中的所有密码信息
    pub fn clear(&mut self) -> PserResult<()> {
        self.psers.inner_mut().clear();
        self.sync_psers()
    }

    /// 根据uuid，返回Pser的引用
    pub fn get_pser(&self, uuid: &str) -> Option<&Pser> {
        self.psers.inner().get(uuid)
    }

    /// 根据uuid，返回Pser的可变引用
    pub fn get_pser_mut(&mut self, uuid: &str) -> Option<&mut Pser> {
        self.psers.inner_mut().get_mut(uuid)
    }
}

fn file_sha2<T: AsRef<Path>>(file: T) -> io::Result<Vec<u8>> {
    let data = std::fs::read(file.as_ref())?;
    let mut hasher: Sha512 = Sha512::new();
    hasher.update(data);
    let res = hasher.finalize();
    Ok(res.to_vec())
}

/// 检查主从密码库文件的sha2值，判断主从密码库的数据是否一致
fn check_db_sha2() {
    let home_sha2 = file_sha2(&*DB_FILE_HOME).unwrap_or_else(|e| panic!("无法读取主密码库: {}", e));
    let cur_sha2 = file_sha2(&*DB_FILE_CUR).unwrap_or_else(|e| panic!("无法读取从密码库: {}", e));
    if home_sha2 != cur_sha2 {
        let mut str = String::new();
        str.push_str("主、从密码库文件数据不一致.\n");
        str.push_str("如果要采取主密码库，则通过该程序删除从密码库文件，然后再次运行.\n");
        str.push_str("如果要采取从密码库，则通过该程序删除主密码库文件，然后再次运行");
        eprintln!("{}", str);
        std::process::exit(1);
    }
}
