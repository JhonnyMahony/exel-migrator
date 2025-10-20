pub type AppResult<T> = std::result::Result<T, AppError>;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("100: {0:?}")]
    FileDialog(&'static str),
    #[error("101: {0:?}")]
    Exel(&'static str),
    #[error("102: {0:?}")]
    Db(&'static str),
    #[error("103: {0:?}")]
    Config(&'static str),
    #[error("104: {0:?}")]
    MT(&'static str),
    #[error("105: Невдалося під'єднатися до бази данних")]
    DbConnErr,
    #[error("105: Невадолося з'єднатися з metabase")]
    MTConnection(#[from] reqwest::Error),
}
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
