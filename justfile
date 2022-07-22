
root := justfile_directory()

install:
    #!/bin/sh
    cd {{root}}
    cargo clean
    rm -rf chesapeake-0.1.0
    mkdir chesapeake-0.1.0
    cp -r src chesapeake-0.1.0
    cp Cargo.toml chesapeake-0.1.0
    cp Cargo.lock chesapeake-0.1.0
    tar czf ../chesapeake-0.1.0.tar.gz chesapeake-0.1.0
    mv ../chesapeake-0.1.0.tar.gz .
    makepkg -g >> PKGBUILD
    makepkg -si


clean:
    cargo clean
    rm -rf logs
    rm -rf pkg
    rm -f *.tar.gz
    rm -f *.pkg.tar.zst
    rm -rf chesapeake-*