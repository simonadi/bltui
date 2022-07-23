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
sha256sums=('49ab1b965a7368543375ba0fe78b9f1ec2c2db3343afb053de783dd5d89cd0aa')

sha256sums=('c407ddd9e29a8cabc143e75aefd8cc930d8f30f960ec884772ca6d525459be5d')
sha256sums=('a3a3eebd021e3a124db85704e8d3c3eea508a75320fa5e9a16ededb3e3ab62a1')
sha256sums=('2c9061335125567eac07b56818b273d13a7f343e9d1362c6ca9049e3c1289b62')
