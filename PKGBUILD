pkgname=bltui
pkgver=0.1.0
pkgrel=1
pkgdesc="Bluetooth TUI"
arch=("x86_64")
license=("custom")
url="https://github.com/clapmytrapp/bltui"
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
    install -Dm0755 -t "$pkgdir/usr/bin/" "target/release/$pkgname"
}

sha256sums=('49ab1b965a7368543375ba0fe78b9f1ec2c2db3343afb053de783dd5d89cd0aa')