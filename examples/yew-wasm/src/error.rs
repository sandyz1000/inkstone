use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ApiError {
    #[error("Error in loading PDF docs, {0}")]
    PdfLoadError(String),
    
    #[error("Error in rendering PDF docs, {0}")]
    PdfViewError(String)
}
