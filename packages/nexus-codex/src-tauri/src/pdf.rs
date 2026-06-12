use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Result};

/// Extract plain text from a PDF.
///
/// First attempts the pure-Rust `pdf-extract` crate. If that fails (encrypted or
/// corrupt PDFs are common), falls back to the `pdftotext -layout <path> -` CLI if
/// it is available on the system. If both approaches fail, the original error is
/// returned.
pub fn extract_text(path: &Path) -> Result<String> {
    match pdf_extract::extract_text(path) {
        Ok(text) if !text.trim().is_empty() => Ok(text),
        Ok(_) => {
            // Extraction succeeded but produced no text — try the CLI fallback.
            match pdftotext_fallback(path) {
                Ok(text) => Ok(text),
                Err(_) => Ok(String::new()),
            }
        }
        Err(primary_err) => match pdftotext_fallback(path) {
            Ok(text) => Ok(text),
            Err(fallback_err) => Err(anyhow!(
                "pdf-extract failed ({primary_err}); pdftotext fallback failed ({fallback_err})"
            )),
        },
    }
}

/// Run `pdftotext -layout <path> -` and capture stdout.
fn pdftotext_fallback(path: &Path) -> Result<String> {
    let output = Command::new("pdftotext")
        .arg("-layout")
        .arg(path)
        .arg("-")
        .output()
        .map_err(|e| anyhow!("could not invoke pdftotext: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("pdftotext exited with error: {}", stderr.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
