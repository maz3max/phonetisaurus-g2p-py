/*

Adapted from https://github.com/lastleon/phonetisaurus-g2p-rs

MIT License

Copyright (c) 2025 lastleon

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

*/

use anyhow::{Context, Result, anyhow};
use rustfst::algorithms::compose;
use rustfst::prelude::*;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Debug)]
/// Result of a phonemization.
pub struct PhonetizationResult {
    /// Phonemes produced during phonemization.
    pub phonemes: String,
    /// Negative log likelihood of phonemes, lower is better.
    pub neg_log_score: f32,
}

#[derive(Clone, Debug)]
/// Phonemizer struct.
pub struct PhonetisaurusModel {
    /// The trained FST.
    ///
    /// It is wrapped inside a smart pointer, since the FST needs to be cloned for each new phonemization.
    /// Arc instead of Rc is used in order to provide thread safety, so that parallel phonemization is possible.
    trained_fst: Arc<VectorFst<TropicalWeight>>,
}

impl TryFrom<&Path> for PhonetisaurusModel {
    type Error = anyhow::Error;

    /// Create a new phonemizer from a phonetisaurus model file.
    fn try_from(model_path: &Path) -> std::result::Result<Self, Self::Error> {
        Ok(PhonetisaurusModel {
            trained_fst: Arc::new(VectorFst::<TropicalWeight>::read(model_path)?),
        })
    }
}

impl TryFrom<&[u8]> for PhonetisaurusModel {
    type Error = anyhow::Error;

    /// Create a new phonemizer from a binary of a phonetisaurus model.
    /// Typically, this would be used with the include_bytes! macro.
    fn try_from(model_binary: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(PhonetisaurusModel {
            trained_fst: Arc::new(VectorFst::<TropicalWeight>::load(model_binary)?),
        })
    }
}

impl PhonetisaurusModel {
    /// Phonemize a word with the phonetisaurus FST model.
    pub fn phonemize_word(&self, word: &str) -> Result<PhonetizationResult> {
        // ACCEPTOR
        let input_sequence: Vec<Label> = self.encode_as_labels(word)?;
        let input_fst = self.create_input_fst(&input_sequence)?;

        // COMPOSE
        // NOTE: The weird type annotation is needed, as Rust doesn't know which Borrow<_> impl
        // to use for the second FST. The impls for both Arc<_> and VectorFst<_> are possible
        // (as far as I understand), and we need to use the second one, so VectorFst<_> needs to
        // be specified as F2. For reference, the full type annotation would be:
        //      W:  TropicalWeight,
        //      F1: VectorFst<TropicalWeight>,
        //      F2: VectorFst<TropicalWeight>,
        //      F3: VectorFst<TropicalWeight>,
        //      B1: VectorFst<TropicalWeight>,
        //      B2: Arc<VectorFst<TropicalWeight>>,
        let composed_fst: VectorFst<TropicalWeight> =
            compose::compose::<_, _, VectorFst<TropicalWeight>, _, _, _>(
                input_fst,
                self.trained_fst.clone(),
            )?;

        // TRANSFORM TO PHONEMES (ITERATE SHORTEST PATH)
        // WARNING: rustfst's shortest_path does not find the shortest paths, phonetisaurus finds better ones
        let shortest_fst: VectorFst<_> = shortest_path(&composed_fst)?;

        let shortest_path = shortest_fst.paths_iter().collect::<Vec<_>>();
        let shortest_path = shortest_path.first().ok_or(anyhow!(
            "Transcription failed: No shortest path found in FST. This should not be possible."
        ))?;
        // only one path should exist, because fst was converted to shortest path fst.

        let osyms = shortest_fst.output_symbols().ok_or(anyhow!(
            "No output symbol table found in loaded FST model, but one is needed."
        ))?;

        // "_" symbols need to be skipped
        // "|" in symbols needs to be removed
        let phonemes = shortest_path
            .olabels
            .iter()
            .filter_map(|&label| {
                if let Some(symbol) = osyms.get_symbol(label) {
                    if symbol == "_" {
                        return None;
                    }

                    Some(Ok(symbol))
                } else {
                    Some(Err(anyhow!(
                        "Symbol for label {} not found in output symbol table",
                        label
                    )))
                }
            })
            .collect::<Result<Vec<&str>>>()?
            .join(" ")
            .replace("|", "");

        Ok(PhonetizationResult {
            phonemes,
            neg_log_score: *shortest_path.weight.value(),
        })
    }

    fn encode_as_labels(&self, word: &str) -> Result<Vec<Label>> {
        let isyms = self.trained_fst.input_symbols().ok_or(anyhow!(
            "No input symbol table found in loaded FST model, but one is needed."
        ))?;
        let mut input_sequence: Vec<Label> = Vec::new();

        // TODO/WARNING: Inputs are not always ASCII, so this can break!
        for ch in word.chars() {
            if let Some(sym) = isyms.get_label(ch.to_string()) {
                input_sequence.push(sym);
            } else {
                return Err(anyhow!(
                    "Symbol {} not found in symbol table. Most likely, the FST was not trained with this symbol.",
                    ch
                ));
            }
        }

        Ok(input_sequence)
    }

    fn create_input_fst(&self, input_sequence: &Vec<Label>) -> Result<VectorFst<TropicalWeight>> {
        let mut input_fst: VectorFst<TropicalWeight> = VectorFst::new();
        let mut state = input_fst.add_state();
        input_fst.set_start(state)?;

        for &sym in input_sequence {
            let next_state = input_fst.add_state();
            input_fst.add_tr(state, Tr::new(sym, sym, TropicalWeight::one(), next_state))
                .context("Constructing acceptor FST from input word failed, new transition could not be added.")?;
            state = next_state;
        }
        input_fst.set_final(state, TropicalWeight::one()).context(
            "Constructing acceptor FST from input word failed, final state could not be set.",
        )?;

        Ok(input_fst)
    }
}
