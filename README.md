# phonetisaurus-g2p-py

A Python wrapper for Phonetisaurus FST-based grapheme-to-phoneme (G2P) conversion. Based on [phonetisaurus-g2p-rs](https://github.com/lastleon/phonetisaurus-g2p-rs).

## Installation

### From source

1. Install maturin: `pip install maturin`
2. Build and install: `maturin develop`

### Building wheels

```bash
maturin build --release
```

## Usage

### Python API

```python
import phonetisaurus_g2p_py

# Load a model
model = phonetisaurus_g2p_py.PhonetisaurusModel("path/to/model.fst")

# Phonemize a word
result = model.phonemize_word("hello")
print(f"Phonemes: {result.phonemes}")
print(f"Score: {result.neg_log_score}")
```

### Command Line

The original CLI is still available:

```bash
cargo run -- path/to/model.fst "hello"
```

## Model Format

This package expects Phonetisaurus FST models in the standard format. You can train your own models using the Phonetisaurus toolkit.

## Requirements

- Python 3.8+
- Rust (for building from source)

## License

MIT
