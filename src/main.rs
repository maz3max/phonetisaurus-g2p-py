use anyhow::Result;
use clap::Parser;
use std::path::Path;

mod phonetisaurus;
use phonetisaurus::*;

/// A command-line tool for phonemizing words using Phonetisaurus FST models
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the Phonetisaurus FST model file
    model_path: String,
    
    /// Word to phonemize
    word: String,
}

fn main() {
    // Parse command-line arguments using clap
    let cli = Cli::parse();

    // Load the model from the specified path
    let phonemizer: PhonetisaurusModel = match PhonetisaurusModel::try_from(Path::new(&cli.model_path)) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to load model from '{}': {}", cli.model_path, e);
            std::process::exit(1);
        }
    };

    // Phonemize the word
    match phonemizer.phonemize_word(&cli.word) {
        Ok(result) => {
            println!("Nofabet: {}", result.phonemes);
        }
        Err(e) => {
            eprintln!("Failed to phonemize word: {}", e);
            std::process::exit(1);
        }
    }
}
