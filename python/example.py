#!/usr/bin/env python3
"""
Example usage of the phonetisaurus_g2p_py Python package
"""

import sys
from pathlib import Path
import phonetisaurus_g2p_py

def main():
    if len(sys.argv) != 3:
        print("Usage: python example.py <model_path> <word>")
        sys.exit(1)
    
    model_path = sys.argv[1]
    word = sys.argv[2]
    
    try:
        # Load the model
        model = phonetisaurus_g2p_py.PhonetisaurusModel(model_path)
        
        # Phonemize the word
        result = model.phonemize_word(word)
        
        print(f"Word: {word}")
        print(f"Phonemes: {result.phonemes}")
        print(f"Score: {result.neg_log_score}")
        
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
