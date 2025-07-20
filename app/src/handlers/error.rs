#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Could not update UI")]
    UIConfig,
    #[error("Could not find window")]
    Window,
    #[error("An error ocurred whilst executing query")]
    Database,
    #[error("An error ocurred whilst doing filesystem operation")]
    FileSystem,
    #[error("Could not spawn LOA process")]
    LoaProcessSpawn,
    #[error("Could not unload windivert driver")]
    WindiverUnload,
    #[error("Could not set start on boot")]
    SetStartOnBoot,
    #[error("Could not send event")]
    Emit
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}