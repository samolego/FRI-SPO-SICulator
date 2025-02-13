pub trait Error {
    fn message(&self) -> String;
}

#[derive(Debug)]
pub struct RegisterError {
    pub index: u8,
}

impl Error for RegisterError {
    fn message(&self) -> String {
        format!("Invalid register index: {}.", self.index)
    }
}

impl From<RegisterError> for String {
    fn from(e: RegisterError) -> Self {
        format!("Invalid register index {}.", e.index)
    }
}
