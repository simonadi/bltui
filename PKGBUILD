pkgname=chesapeake
pkgver=0.1.0
pkgrel=1
pkgdesc="Bluetooth TUI"
arch=("x86_64")
license=("custom")
url="https://github.com/clapmytrapp/chesapeake"
depends=("bluez")
makedepends=("cargo")
# source=("${pkgname}-${pkgver}.tar.gz::https://github.com/clapmytrapp/${pkgname}/archive/${pkgver}.tar.gz")
source=("${pkgname}-${pkgver}.tar.gz")

build() {
    cd "${pkgname}-${pkgver}"
    cargo build --release
}

check() {
    cd "${pkgname}-${pkgver}"
    cargo check --release
}

package() {
    cd "${pkgname}-${pkgver}"
    echo "lol"
    install -Dm0755 -t  "$pkgdir/usr/bin/" "target/release/$pkgname"
}

sha256sums=('63df8d5cfdea80ec77b77c45e8560e686aaf424c76bd80235d44110bea13069c')
sha256sums=('a33c8501d956e2edb2d4480ce07f07ec2e3462f5e48a114c95e4975b07ad67e1')
