use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    Io(std::io::Error),
    Display(String),
    SystemInfo(String),
    Multiplexer(Box<dyn std::error::Error + Send + Sync>),
    Config(crate::config::ConfigError),
    Daemon(String),
    ScreenFactory(String),
    ScreenManager(String),
    Application(String),
    Hardware(String),
    Permission(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "I/O error: {}", err),
            AppError::Display(msg) => write!(f, "Display error: {}", msg),
            AppError::SystemInfo(msg) => write!(f, "System info error: {}", msg),
            AppError::Multiplexer(err) => write!(f, "Multiplexer error: {}", err),
            AppError::Config(err) => write!(f, "Configuration error: {}", err),
            AppError::Daemon(msg) => write!(f, "Daemon error: {}", msg),
            AppError::ScreenFactory(msg) => write!(f, "Screen factory error: {}", msg),
            AppError::ScreenManager(msg) => write!(f, "Screen manager error: {}", msg),
            AppError::Application(msg) => write!(f, "Application error: {}", msg),
            AppError::Hardware(msg) => write!(f, "Hardware error: {}", msg),
            AppError::Permission(msg) => write!(f, "Permission error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(err) => Some(err),
            AppError::Multiplexer(err) => Some(err.as_ref()),
            AppError::Config(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<crate::config::ConfigError> for AppError {
    fn from(err: crate::config::ConfigError) -> Self {
        AppError::Config(err)
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> Self {
        AppError::SystemInfo(format!("Parse error: {}", err))
    }
}

impl From<std::num::ParseFloatError> for AppError {
    fn from(err: std::num::ParseFloatError) -> Self {
        AppError::SystemInfo(format!("Parse error: {}", err))
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::SystemInfo(format!("System error: {}", err))
    }
}

// Convenience type alias
pub type Result<T> = std::result::Result<T, AppError>;

// Helper functions for creating errors
impl AppError {
    pub fn display_init(msg: &str) -> Self {
        AppError::Display(msg.to_string())
    }

    pub fn system_info(msg: &str) -> Self {
        AppError::SystemInfo(msg.to_string())
    }

    pub fn daemon(msg: &str) -> Self {
        AppError::Daemon(msg.to_string())
    }

    #[allow(dead_code)]
    pub fn screen_factory(msg: &str) -> Self {
        AppError::ScreenFactory(msg.to_string())
    }

    #[allow(dead_code)]
    pub fn screen_manager(msg: &str) -> Self {
        AppError::ScreenManager(msg.to_string())
    }

    #[allow(dead_code)]
    pub fn application(msg: &str) -> Self {
        AppError::Application(msg.to_string())
    }

    #[allow(dead_code)]
    pub fn hardware(msg: &str) -> Self {
        AppError::Hardware(msg.to_string())
    }

    #[allow(dead_code)]
    pub fn permission(msg: &str) -> Self {
        AppError::Permission(msg.to_string())
    }

    #[allow(dead_code)]
    pub fn multiplexer<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        AppError::Multiplexer(Box::new(err))
    }
}

// Helper trait for converting display errors
#[allow(dead_code)]
pub trait DisplayErrorExt<T> {
    fn display_err(self, context: &str) -> Result<T>;
}

impl<T, E: fmt::Debug> DisplayErrorExt<T> for std::result::Result<T, E> {
    fn display_err(self, context: &str) -> Result<T> {
        self.map_err(|e| AppError::display_init(&format!("{}: {:?}", context, e)))
    }
}