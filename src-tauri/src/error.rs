use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("验证错误: {0}")]
    Validation(String),

    #[error("方案不存在: {0}")]
    NotFound(String),

    #[error("方案名称已存在: {0}")]
    DuplicateName(String),

    #[error("网络操作失败: {0}")]
    Network(String),

    #[error("权限不足: {0}")]
    PermissionDenied(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
