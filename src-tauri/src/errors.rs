pub type AppResult<T> = std::result::Result<T, AppError>;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Filedialog: {0:?}")]
    FileDialog(&'static str),
    #[error("Exel: {0:?}")]
    Exel(&'static str),
    #[error("Exel: {0:?}")]
    Db(&'static str),
    #[error("Config: {0:?}")]
    Config(&'static str),
    #[error("Metabase: {0:?}")]
    MT(&'static str),
    #[error("Невдалося під'єднатися до бази данних")]
    DbConnErr,
    #[error("Невадолося з'єднатися з metabase")]
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
