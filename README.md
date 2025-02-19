# beet

<div align="center">
  <p>
    <strong>Tools for developing reactive structures.</strong>
  </p>
  <p>
    <a href="https://crates.io/crates/beet"><img src="https://img.shields.io/crates/v/beet.svg?style=flat-square" alt="Crates.io version" /></a>
    <a href="https://crates.io/crates/beet"><img src="https://img.shields.io/crates/d/beet.svg?style=flat-square" alt="Download" /></a>
    <a href="https://docs.rs/beet"><img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>
  </p>
  <h3>
    <!-- <a href="https://docs.rs/beet">Guidebook</a> -->
    <!-- <span> | </span> -->
    <a href="https://docs.rs/beet">API Docs</a>
    <!-- <span> | </span>
    <a href="https://mrchantey.github.io/beet/other/contributing.html">Contributing</a> -->
  </h3>
</div>

Beet is a collection of crates for authoring and running web pages, games and AI behaviors. Your mileage may vary depending on the crate of interest:

- 🦢 documented and tested
- 🐣 docs and tests are incomplete
- 🐉 highly experimental, here be dragons

| Crate                                            | Status | Description                                                       |
| ------------------------------------------------ | ------ | ----------------------------------------------------------------- |
| [`beet_flow`](crates/beet_flow/Cargo.toml)       | 🦢      | Scenes-as-control-flow bevy library for behavior trees etc        |
| [`beet_spatial`](crates/beet_spatial/Cargo.toml) | 🐣      | Extend `beet_flow` with spatial behaviors like steering           |
| [`beet_ml`](crates/beet_ml/Cargo.toml)           | 🐉      | Extend `beet_flow` with machine learning using `candle`           |
| [`beet_sim`](crates/beet_sim/Cargo.toml)         | 🐉      | Extend `beet_flow` with generalized simulation tooling like stats |
| [`beet_rsx`](crates/beet_rsx/Cargo.toml)         | 🐉      | Exploration of authoring tools for html and bevy                  |
| [`beet_router`](crates/beet_router/Cargo.toml)   | 🐉      | File based router for web docs                                    |

The `beet` crate serves as a base crate that re-exports any combination of sub-crates according to feature flags, much like the `bevy` crate structure.

## Bevy Versions

| `bevy` | `beet` |
| ------ | ------ |
| 0.15   | 0.0.4  |
| 0.14   | 0.0.2  |
| 0.12   | 0.0.1  |