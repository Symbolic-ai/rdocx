//! rdocx CLI — "jq for DOCX"
//!
//! Inspect, convert, diff, and manipulate DOCX files from the command line.

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "rdocx", version, about = "CLI tool for DOCX files")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print document structure: paragraph/table count, styles, images, metadata
    Inspect {
        /// Path to the DOCX file
        file: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Extract plain text from a DOCX file
    Text {
        /// Path to the DOCX file
        file: PathBuf,
    },
    /// Convert DOCX to another format (pdf, html, md, png, jpeg, tiff)
    Convert {
        /// Path to the DOCX file
        file: PathBuf,
        /// Output format: pdf, html, md, png, jpeg, tiff
        #[arg(long, short = 't')]
        to: String,
        /// Output file path (defaults to input with new extension)
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
        /// DPI for image rendering (default: 150)
        #[arg(long, default_value = "150")]
        dpi: u32,
        /// Directory containing font files (.ttf/.otf) to use for PDF rendering
        #[arg(long)]
        font_dir: Option<PathBuf>,
        /// One-based page range for image output, such as 1,3-5
        #[arg(long)]
        pages: Option<String>,
        /// JPEG quality from 1 through 100
        #[arg(long, default_value = "90")]
        quality: u8,
        /// Preserve unpainted PNG pixels as transparent
        #[arg(long)]
        transparent: bool,
    },
    /// Structural diff between two DOCX files
    Diff {
        /// First DOCX file
        file_a: PathBuf,
        /// Second DOCX file
        file_b: PathBuf,
    },
    /// Replace placeholders in a DOCX file
    Replace {
        /// Path to the DOCX file
        file: PathBuf,
        /// Placeholder string
        #[arg(long, short = 'p')]
        placeholder: String,
        /// Replacement value
        #[arg(long, short = 'v')]
        value: String,
        /// Output file path
        #[arg(long, short = 'o')]
        output: PathBuf,
    },
    /// Validate OOXML conformance
    Validate {
        /// Path to the DOCX file
        file: PathBuf,
    },
    /// Render pages to image files
    Render {
        /// Path to the DOCX file
        file: PathBuf,
        /// Output directory (defaults to current directory)
        #[arg(long, short = 'o')]
        output_dir: Option<PathBuf>,
        /// DPI resolution (default: 150)
        #[arg(long, default_value = "150")]
        dpi: f64,
        /// Render only a specific page (0-based index)
        #[arg(long, conflicts_with = "pages")]
        page: Option<usize>,
        /// One-based page range, such as 1,3-5
        #[arg(long)]
        pages: Option<String>,
        /// Output format: png, jpeg, tiff
        #[arg(long, default_value = "png")]
        format: String,
        /// JPEG quality from 1 through 100
        #[arg(long, default_value = "90")]
        quality: u8,
        /// Preserve unpainted PNG pixels as transparent
        #[arg(long)]
        transparent: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // `validate` is the one command whose exit status carries a verdict, so it
    // is dispatched separately from the commands that only report errors.
    if let Command::Validate { file } = &cli.command {
        match commands::validate(file) {
            Ok(true) => return,
            Ok(false) => process::exit(1),
            Err(e) => {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }
    }

    let result = match cli.command {
        Command::Inspect { file, json } => commands::inspect(&file, json),
        Command::Text { file } => commands::text(&file),
        Command::Convert {
            file,
            to,
            output,
            dpi,
            font_dir,
            pages,
            quality,
            transparent,
        } => commands::convert(
            &file,
            &to,
            output.as_deref(),
            dpi,
            font_dir.as_deref(),
            commands::ImageOptions {
                pages: pages.as_deref(),
                quality,
                transparent,
            },
        ),
        Command::Diff { file_a, file_b } => commands::diff(&file_a, &file_b),
        Command::Replace {
            file,
            placeholder,
            value,
            output,
        } => commands::replace(&file, &placeholder, &value, &output),
        // Handled above so its exit code can reflect the verdict.
        Command::Validate { .. } => unreachable!(),
        Command::Render {
            file,
            output_dir,
            dpi,
            page,
            pages,
            format,
            quality,
            transparent,
        } => commands::render(
            &file,
            output_dir.as_deref(),
            dpi,
            commands::RenderOptions {
                page,
                pages: pages.as_deref(),
                format: &format,
                quality,
                transparent,
            },
        ),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
