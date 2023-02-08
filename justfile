
root := justfile_directory()

install:
    #!/bin/sh
    cd {{root}}
    just clean
    mkdir -p temp/bltui-0.1.0
    cp -r src temp/bltui-0.1.0
    cp Cargo.toml temp/bltui-0.1.0
    cp Cargo.lock temp/bltui-0.1.0
    ln ./PKGBUILD ./temp/PKGBUILD
    cd temp
    tar czf bltui-0.1.0.tar.gz bltui-0.1.0
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
    rm -rf bltui-*

check:
    #!/bin/sh
    cargo check -p bltui
    printf '%*s\n' "${COLUMNS:-$(tput cols)}" '' | tr ' ' -
    cargo clippy -p bltui
    printf '%*s\n' "${COLUMNS:-$(tput cols)}" '' | tr ' ' -
    cargo fmt

commit:
    convco commit

test:
    cargo nextest run