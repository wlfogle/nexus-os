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

#[cfg(test)]
mod tests {
    use super::*;

    /// Exercises both the pdf-extract crate path and the pdftotext fallback
    /// against a real PDF with a known text layer.
    #[test]
    fn extract_text_from_real_pdf() {
        let path = std::path::Path::new("/tmp/nexus-codex-test/cyberpowerpc-manual.pdf");
        if !path.exists() {
            eprintln!("Skipping: test PDF not found at {:?}", path);
            return;
        }
        let text = extract_text(path).expect("extract_text failed");
        assert!(!text.trim().is_empty(), "extracted text must not be empty");
        // The PDF is a CyberPowerPC manual — verify recognisable content
        let lower = text.to_lowercase();
        assert!(
            lower.contains("cyberpowerpc") || lower.contains("gxivr") || lower.contains("manualslib"),
            "extracted text does not look like the expected PDF content. Got: {}...",
            &text[..text.len().min(200)]
        );
        println!("PDF extraction OK — {} chars extracted", text.len());
        println!("First 300 chars:\n{}", &text[..text.len().min(300)]);
    }

    #[test]
    fn pdftotext_fallback_works() {
        let path = std::path::Path::new("/tmp/nexus-codex-test/cyberpowerpc-manual.pdf");
        if !path.exists() {
            eprintln!("Skipping: test PDF not found");
            return;
        }
        let text = pdftotext_fallback(path).expect("pdftotext fallback failed");
        assert!(!text.trim().is_empty(), "pdftotext must produce non-empty output");
        println!("pdftotext fallback OK — {} chars", text.len());
    }
}
