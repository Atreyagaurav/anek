# Maintainer: Gaurav Atreya <allmanpride@gmail.com>
pkgname=anek
pkgver=0.3.1
pkgrel=1
pkgdesc="Tool to run commands based on a templates"
arch=("x86_64")
url="https://github.com/Atreyagaurav/anek"
license=('GPL3')
groups=()
depends=()
makedepends=('git' 'cargo')
provides=("${pkgname}")
conflicts=("${pkgname}")
replaces=()
backup=()
options=()
install=
source=("${pkgname}-${pkgver}.tar.gz::https://github.com/Atreyagaurav/anek/archive/refs/tags/v${pkgver}.tar.gz")
noextract=()
md5sums=('SKIP')

build() {
	echo "$srcdir/${pkgname}-${pkgver}"
	cd "$srcdir/${pkgname}-${pkgver}"
	make
}

package() {
    cd "$srcdir/${pkgname}-${pkgver}"
    mkdir -p "$pkgdir/usr/bin"
    cp target/release/anek "$pkgdir/usr/bin/anek"
    mkdir -p "$pkgdir/usr/share/bash-completion/completions"
    cp completions/bash-completions.sh "$pkgdir/usr/share/bash-completion/completions/anek"
}
