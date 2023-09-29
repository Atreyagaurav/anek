# Maintainer: Gaurav Atreya <allmanpride@gmail.com>
pkgname=anek
pkgver=0.5.2
pkgrel=1
pkgdesc="Tool to run commands based on a templates"
arch=('x86_64')
url="https://github.com/Atreyagaurav/${pkgname}"
license=('GPL3')
depends=('gcc-libs')
makedepends=('rust' 'cargo' 'git')

build() {
	cargo build --release
}

package() {
    cd "$srcdir"
    mkdir -p "$pkgdir/usr/bin"
    cp "../target/release/${pkgname}" "$pkgdir/usr/bin/${pkgname}"
    mkdir -p "$pkgdir/usr/share/bash-completion/completions"
    cp "../completions/bash-completions.sh" "$pkgdir/usr/share/bash-completion/completions/${pkgname}"
    mkdir -p "$pkgdir/usr/share/fish/vendor_completions.d/"
    "../target/release/${pkgname}" completions --fish > "$pkgdir/usr/share/fish/vendor_completions.d/${pkgname}.fish"
    mkdir -p "$pkgdir/usr/share/zsh/site-functions/"
    "../target/release/${pkgname}" completions --zsh > "$pkgdir/usr/share/zsh/site-functions/_${pkgname}"
}
