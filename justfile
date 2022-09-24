
root := justfile_directory()

install:
    #!/bin/sh
    cd {{root}}
    cargo clean
    rm -rf bluetui-0.1.0
    mkdir bluetui-0.1.0
    cp -r src bluetui-0.1.0
    cp Cargo.toml bluetui-0.1.0
    cp Cargo.lock bluetui-0.1.0
    tar czf ../bluetui-0.1.0.tar.gz bluetui-0.1.0
    mv ../bluetui-0.1.0.tar.gz .
    makepkg -g >> PKGBUILD
    makepkg -si

fmt:
    cargo +nightly fmt

run:
    cargo run --release

clean:
    cargo clean
    rm -rf logs
    rm -rf pkg
    rm -f **/*.tar.gz
    rm -f *.pkg.tar.zst
    rm -rf src/bluetui-*
    rm -rf bluetui-*

check:
    #!/bin/sh
    cargo check -p bluetui
    printf '%*s\n' "${COLUMNS:-$(tput cols)}" '' | tr ' ' -
    cargo clippy -p bluetui
    printf '%*s\n' "${COLUMNS:-$(tput cols)}" '' | tr ' ' -
    cargo fmt