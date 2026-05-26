#[derive(Debug, Clone, Copy, Default)]
pub enum LogLevel {
    #[default]
    None,
    Basic,
    Headers,
    Full,
}