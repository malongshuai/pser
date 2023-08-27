pub mod add_cmd;
pub mod gen_cmd;

use clap::{Parser, Subcommand, ValueEnum};

pub use add_cmd::InsertCmd;
pub use gen_cmd::GenPasswdCmd;

/// 管理密码、生成随机密码、导入导出密码
/// 
/// 在各个子命令中，任何一个需要提供主密码的地方，都可以通过设置环境变量`PSER_PASSWD`来提供，
/// 如果没有提供，在需要主密码的地方，将交互式提示你输入密码
#[derive(Debug, Parser)]
pub struct Opts {
    #[command(subcommand)]
    pub cmds: Cmds,
}

#[derive(Debug, Subcommand)]
pub enum Cmds {
    Gen(GenPasswdCmd),
    /// 初始化(创建)密码库，
    /// 如果当前已经存在密码库，则仅仅只是验证输入的密码是否正确
    Init,
    Query(QueryCmd),
    Insert(InsertCmd),
    Rm(RmCmd),
    Reset(ResetCmd),
    Import(ImportCmd),
    Export(ExportCmd),
}

/// 搜索密码库中的密码信息。搜索时，只根据密码的所属url或所属desc进行搜索(忽略大小写)
///
/// 例如, 搜索google的账号信息：$0 query "google"
#[derive(Debug, Parser)]
pub struct QueryCmd {
    /// 简略输出搜索结果：只输出uuid、url、username、password
    #[clap(short, long)]
    pub short: bool,
    /// 指定搜索关键字，如果省略，则列出密码库中所有信息
    pub str: Option<String>,
}

/// 删除或清空密码信息
///
/// 需指定UUID，如果不知道UUID，可先通过query子命令查询
#[derive(Debug, Parser)]
pub struct RmCmd {
    /// 指定UUID(前缀)来选择删除哪个密码，
    ///
    /// - 可通过逗号分隔多个UUID(前缀)
    /// 
    /// - 如果UUID指定为特殊值`all`(不区分大小写)，则清空密码库中的所有密码信息
    ///
    /// - 如果UUID指定为特殊值`remove`(不区分大小写)，则删除所有密码库文件(主、从密码文件都删除)
    pub uuid: String,
}

/// 重置主密码
#[derive(Debug, Parser)]
pub struct ResetCmd {
    /// 指定新密码
    #[clap(short, long = "new")]
    pub new_passwd: String,
}

/// 导入密码信息
///
/// 只能导入到已经存在的密码库(将会在当前密码库中添加导入数据中的每一条密码信息)，因此如果还没有密码库，应当先初始化
#[derive(Debug, Parser)]
pub struct ImportCmd {
    /// 指定要导入的密码信息源文件，如果省略该选项，则从标准输入中读取
    ///
    /// 需注意，如果导入的是csv文件格式，则只允许是来自浏览器导出的csv文件，要求各字段的名称固定为：
    ///
    /// name,url,username,password,comment
    ///
    /// 但各字段顺序可随意，且name和comment字段可以省略。name字段对应密码库中的"所属(desc)"字段。
    ///
    /// 如果不是字段名称不对，应修改csv文件第一行的csv头部，将其对应为这几个字段名。
    ///
    /// 如果是json文件，则是来自本程序 export 子命令的到处数据，只要到处后未曾修改过文件，则没有格式限制。
    #[clap(short, long)]
    pub input: Option<String>,

    /// 指定导入文件的类型，只接受两种值："csv"和"json"
    #[clap(short, long = "src-type", value_enum)]
    pub src_type: ImportSrcType,
}

#[derive(Debug, ValueEnum, Copy, Clone)]
pub enum ImportSrcType {
    Csv,
    Json,
}

/// 导出密码信息为json格式
#[derive(Debug, Parser)]
pub struct ExportCmd {
    /// 指定导出的目标文件，省略该选项将输出到标准输出
    #[clap(short, long)]
    pub output: Option<String>,
}
