# Normal form finder trait

[![CI](https://github.com/timothee-haudebourg/normal-form/workflows/CI/badge.svg)](https://github.com/timothee-haudebourg/normal-form/actions)
[![Crate informations](https://img.shields.io/crates/v/normal-form.svg?style=flat-square)](https://crates.io/crates/normal-form)
[![License](https://img.shields.io/crates/l/normal-form.svg?style=flat-square)](https://github.com/timothee-haudebourg/normal-form#license)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/normal-form)

<!-- cargo-rdme start -->

This library provides a simple method to find the normal/canonical form
of a structure, as long as it implements the provided `Normalize` trait.
It is an implementation of *Practical graph isomorphism, II*
[[McKay 2013]](https://arxiv.org/pdf/1301.1493.pdf) and heavily inspired by
the [canonical-form](https://crates.io/crates/canonical-form) crate with the
addition of caching and associated abstraction types.

<!-- cargo-rdme end -->

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
