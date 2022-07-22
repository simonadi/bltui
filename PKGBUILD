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

sha256sums=('4427e1e06e64c2a9150e724ad460af837d9825df21142a303b80548e82bfdac9')
