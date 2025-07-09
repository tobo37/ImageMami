# Gemma Training Environment

This directory holds a Python virtual environment and helper files to start training [Gemma](https://github.com/google-deepmind/gemma).

## Setup

1. Ensure Python 3.12 is installed.
2. Activate the provided virtual environment:
   ```bash
   source venv/bin/activate
   ```
3. Upgrade pip and install Gemma with its dependencies following the [JAX installation guide](https://jax.readthedocs.io/en/latest/installation.html):
   ```bash
   pip install gemma
   ```

## Training Example

`train_example.py` shows a minimal fine‑tuning loop adapted from the Gemma documentation. Run it from the activated environment:

```bash
python train_example.py
```
