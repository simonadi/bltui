# bltui 

A Bluetooth device managing TUI

[![Latest Version][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![CI Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/bltui.svg
[crates-url]: https://crates.io/crates/bltui
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/tokio-rs/tokio/blob/master/LICENSE
[actions-badge]: https://github.com/simonadi/bltui/actions/workflows/ci.yml/badge.svg?branch=main
[actions-url]: https://github.com/simonadi/bltui/actions?query=workflow%3ACI+branch%3Amain
---

## Installation

### Cargo

```
cargo install bltui
```

### AUR

```
sudo pacman -S bltui
```

 <!-- - `-d/-dd`: enable debug/trace log level. Recommended to use file logging with it since logger output is small.
 - `-u`: show devices with an unknown name 
 - `-l`: log to file (`$HOME/.bltui/logs`)
 - `-a {ADAPTER}`: adapter -->

## Keybindings

| Key             | Action               |
|-----------------|----------------------|
| `q`             | quit                 |
| `s`             | trigger scanning     |
| `c`             | connect              |
| `d`             | disconnect           |
| `j k`/`down up` | move through devices |

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/simonadi/bltui/blob/master/LICENSE