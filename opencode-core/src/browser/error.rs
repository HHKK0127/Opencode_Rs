use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Chrome binary not found. Set CHROME_BIN or install Chrome.")]
    ChromeNotFound,

    #[error("Chrome launch failed: {0}")]
    ChromeLaunchFailed(String),

    #[error("CDP connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Tab not found in session '{session}'")]
    TabNotFound { session: String },

    #[error("Navigation timeout after {secs}s for URL: {url}")]
    NavigationTimeout { secs: u64, url: String },

    #[error("Element not found: {selector}")]
    ElementNotFound { selector: String },

    #[error("JavaScript evaluation error: {0}")]
    EvaluateError(String),

    #[error("Screenshot error: {0}")]
    ScreenshotError(String),

    #[error("PDF generation error: {0}")]
    PdfError(String),

    #[error("Path traversal detected: {0}")]
    PathTraversal(String),

    #[error("Missing required path for screenshot/PDF")]
    MissingPath,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Browser not running")]
    NotRunning,

    #[error("Max concurrent tabs ({max}) reached")]
    MaxTabsReached { max: usize },
}
