[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/shenjiangqiu/index_permute/rust.yml)](https://github.com/shenjiangqiu/index_permute/actions/workflows/rust.yml)
[![Crates.io Version](https://img.shields.io/crates/v/index_permute)](https://crates.io/crates/index_permute)

# 🌀 index_permute

A minimal, **in-place**, and **non-cloning** array permutation library for Rust.

This crate allows you to **reorder a slice in place by an index array**, even when the element type is **not `Clone` or `Copy`**. It ensures safety via a wrapper type `PermuteIndex` that checks index validity ahead of time.

---

## ✨ Features

- ✅ In-place permutation of non-`Copy`, non-`Clone` data.
- ✅ Memory-safe: no element dropped or cloned during permutation.
- ✅ Index validation: ensures the index is a true permutation (`0..N`).
- ✅ Safe, ergonomic API.
- ✅  `parallel` feature is defined, but it has limited speedup due to it's memory-bound nature. On benchmarks, it shows a 1.3x speedup on a 10 million element array.

---

## Example

```rust
use index_permute::PermuteIndex;
let index = PermuteIndex::try_new(&[2, 0, 1]).unwrap();
let mut data = vec![10, 20, 30];
index_permute::order_by_index_inplace(&mut data, index);
assert_eq!(data, vec![30, 10, 20]);
```

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
index_permute = 0.1
