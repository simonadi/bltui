
root := justfile_directory()

install:
    #!/bin/sh
    cd {{root}}
    just clean
    mkdir -p temp/bluetui-0.1.0
    cp -r bluetui temp/bluetui-0.1.0
    cp -r bluez-async temp/bluetui-0.1.0
    cp -r btleplug temp/bluetui-0.1.0
    cp Cargo.toml temp/bluetui-0.1.0
    cp Cargo.lock temp/bluetui-0.1.0
    ln ./PKGBUILD ./temp/PKGBUILD
    cd temp
    tar czf bluetui-0.1.0.tar.gz bluetui-0.1.0
    sed -i --follow-symlinks '$ d' PKGBUILD
    makepkg -g >> PKGBUILD
    makepkg --force -si
    just clean

fmt:
    cargo +nightly fmt

run *ARGS:
    cargo run --release -- {{ ARGS }}

run-debug:
    cargo run -- -dd -l

clean:
    #cargo clean
    rm -rf logs
    rm -rf temp
    rm -rf pkg
    rm -f **/*.tar.gz
    rm -f *.pkg.tar.zst
    rm -rf src
    rm -rf bluetui-*

check:
    #!/bin/sh
    cargo check -p bluetui
    printf '%*s\n' "${COLUMNS:-$(tput cols)}" '' | tr ' ' -
    cargo clippy -p bluetui
    printf '%*s\n' "${COLUMNS:-$(tput cols)}" '' | tr ' ' -
    cargo fmt