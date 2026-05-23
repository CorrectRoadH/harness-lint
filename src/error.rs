use thiserror::Error;

#[derive(Debug, Error)]
pub enum HarnessError {
    #[error("HL001 config error: {0}")]
    Config(String),
    #[error("HL002 rule pack error: {0}")]
    Pack(String),
    #[error("HL003 rule error: {0}")]
    Rule(String),
    #[error("HL004 Grit error: {0}")]
    Grit(String),
    #[error("HL005 git error: {0}")]
    Git(String),
    #[error("HL006 cache error: {0}")]
    Cache(String),
}

impl HarnessError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Config(_) => "HL001",
            Self::Pack(_) => "HL002",
            Self::Rule(_) => "HL003",
            Self::Grit(_) => "HL004",
            Self::Git(_) => "HL005",
            Self::Cache(_) => "HL006",
        }
    }
}
