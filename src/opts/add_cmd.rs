use clap::Parser;

/// 添加或修改已有密码
///
/// 例如:
///
/// 添加密码(不指定--uuid选项)：
///
/// .    $0 insert --user peny --url google.com
///        --desc google --email peny@qq.com
///        --phone 12343211234 --passwd 'Pass@word'
///        --comment 'card:622218291928312'
///
/// 修改密码(指定--uuid选项)：只修改密码，其它信息不变
///
/// .    $0 insert --uuid asdksld --passwd 'Pass@word'
#[derive(Debug, Parser)]
pub struct InsertCmd {
    /// UUID，指定该选项表示修改已有密码而不是添加密码，
    ///
    /// UUID值可以是前几个字符。如果密码库中找不到指定UUID对应的密码信息，则什么都不做。
    #[clap(short = 'i', long)]
    pub uuid: Option<String>,

    /// 账户名/用户名
    #[clap(short, long)]
    pub user: Option<String>,

    /// 哪个网站的账户(基于域名)
    #[clap(short = 'U', long)]
    pub url: Option<String>,

    /// 什么账户，可以结合url或者和url二选一
    /// 
    /// 例如google的账户，可以记录为`url:google.com`或者`desc:google`，或者两者都记录
    #[clap(short, long)]
    pub desc: Option<String>,

    /// 邮箱，如果有第二邮箱，可使用备注保存，或者使用逗号分隔多个邮箱，
    /// 
    /// 如果是修改操作，首字符使用`+`，将表示追加邮箱信息，否则将覆盖已有邮箱信息。
    #[clap(short, long)]
    pub email: Option<String>,

    /// 联系方式，如果有第二联系方式，可使用备注保存，或者使用逗号分隔多个联系方式，
    /// 
    /// 如果是修改操作，首字符使用`+`，将表示追加联系方式，否则将覆盖已有联系方式。
    #[clap(short = 'P', long)]
    pub phone: Option<String>,

    /// 密码，可以是单个密码，也可以是逗号分隔的多个单词助记词
    #[clap(short, long)]
    pub passwd: Option<String>,

    /// 备注，记录额外信息。
    ///
    /// 如果是修改操作，且首字符使用`+`，将追加备注信息，否则将覆盖已有备注信息。
    /// 
    /// comment中的英文逗号`,`和英文分号`;`在**输出显示**时它们都将显示为换行符
    #[clap(short, long)]
    pub comment: Option<String>,
}
