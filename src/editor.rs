use anyhow::Result;
use retas_core::{Document, Project};
use retas_io::{CelFile, DgaFile, ScsFile, FileReader};
use tracing::info;

pub fn run() -> Result<()> {
    info!("Starting RETAS Studio...");
    
    retas_ui::run().map_err(|e| anyhow::anyhow!("UI error: {}", e))
}

pub fn run_with_file(path: &str) -> Result<()> {
    info!("Opening file: {}", path);

    let extension = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match extension.as_deref() {
        Some("cel") => {
            let cel = CelFile::open(path)?;
            info!("Loaded CEL file: {} ({}x{}, {} frames, {} layers)",
                cel.header.name,
                cel.header.width,
                cel.header.height,
                cel.header.frame_count,
                cel.header.layer_count
            );
            let _doc = cel.to_document();
            retas_ui::run().map_err(|e| anyhow::anyhow!("UI error: {}", e))
        }
        Some("dga") => {
            let dga = DgaFile::open(path)?;
            info!("Loaded DGA file: {} ({}x{}, {} strokes)",
                dga.header.name,
                dga.header.width,
                dga.header.height,
                dga.header.stroke_count
            );
            retas_ui::run().map_err(|e| anyhow::anyhow!("UI error: {}", e))
        }
        Some("scs") => {
            let scs = ScsFile::open(path)?;
            info!("Loaded SCS file: {} ({} layers, {} frames)",
                scs.header.name,
                scs.header.layer_count,
                scs.header.total_frames
            );
            retas_ui::run().map_err(|e| anyhow::anyhow!("UI error: {}", e))
        }
        Some(ext) => {
            anyhow::bail!("Unsupported file format: {}", ext);
        }
        None => {
            anyhow::bail!("Could not determine file type");
        }
    }
}
