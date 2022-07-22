
root := justfile_directory()

install:
    #!/bin/sh
    cargo clean
    tar czf ../chesapeake-0.1.0.tar.gz --directory=.. chesapeake
    mv ../chesapeake-0.1.0.tar.gz .
    makepkg -g >> PKGBUILD
    makepkg -si