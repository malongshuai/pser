use clap::Parser;

/// 生成随机密码
///
/// 注：即便指定了要包含某种字符集，生成的随机密码中可仍然可能会不包含该字符集的字符，
/// 指定包含某种字符集的选项，仅表示生成随机密码时会考虑这类字符集，不代表生成的结果中一定包含该类字符。
/// 可以考虑一次性多个随机密码，从中选择一个或多个。
#[derive(Debug, Parser)]
pub struct GenPasswdCmd {
    /// 是否要考虑大写字母字符集`[A-Z]`
    #[clap(short, long)]
    pub up: bool,

    /// 是否要考虑小写字母字符集`[a-z]`
    #[clap(short, long)]
    pub down: bool,

    /// 是否要考虑数字字符集`[0-9]`
    #[clap(short, long)]
    pub num: bool,

    /// 是否要考虑其它ASCII字符集`)(*&^%$#@!~`
    #[clap(short, long)]
    pub punc: bool,

    /// 生成多个随机密码，每个随机密码一行，默认只生成一个随机密码
    #[clap(short, long, default_value_t = 1)]
    pub cnt: usize,

    /// 密码长度，默认长度为8
    #[clap(default_value_t = 8)]
    pub len: u8,
}
