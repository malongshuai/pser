#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid header")]
    HeaderError,

    #[error(transparent)]
    DecodeError(#[from] bincode::Error),

    /// 数据库相关错误
    #[error(transparent)]
    DBError(#[from] redb::Error),

    /// io错误
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type PserResult<T> = Result<T, Error>;
