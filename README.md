# rsgit
Pure Rust-native implementation of git

[![Tests](https://github.com/rust-git/rsgit/workflows/Tests/badge.svg)](https://github.com/rust-git/rsgit/actions?query=workflow%3ATests)
[![Code coverage](https://codecov.io/gh/rust-git/rsgit/branch/main/graph/badge.svg)](https://codecov.io/gh/rust-git/rsgit)

**This is a very preliminary project.** I'm doing this for fun and as a sandbox to better understand git and its internals and to explore how it feels to create a large software project in Rust. (I'm a relatively new Rust developer, but I am very much enjoying the experience so far.)

As of this writing (October 2020), this project implements a fairly thin subset of git's core infrastructure. It's not even close to ready to be used. I am favoring code quality (careful architecture and testing infrastructure) over rapid growth of the project. I do hope to eventually produce a reasonably full implementation of git, but I make no guarantee that this will happen.

My employer is aware of this project and accepts my participation in it, but it is not officially supported by anyone. I do this on my own time and any development or support happens only as my personal time allows.

## Organization

There are multiple crates in this repo:

* `rsgit_core` implements the core git data model in an abstract sense.
* `rsgit_on_disk` implements above using the same file/folder layout as traditional command-line git.
* `rsgit_cli` implements a subset of the traditional git command-line interface using the `core` and `on_disk` crates.
