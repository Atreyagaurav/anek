# Maintainer: Gaurav Atreya <allmanpride@gmail.com>
pkgname=anek
pkgver=0.6.1
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
    "../target/release/${pkgname}" -q completions --fish > "$pkgdir/usr/share/fish/vendor_completions.d/${pkgname}.fish"
    mkdir -p "$pkgdir/usr/share/zsh/site-functions/"
    "../target/release/${pkgname}" -q completions --zsh > "$pkgdir/usr/share/zsh/site-functions/_${pkgname}"
    mkdir -p "$pkgdir/usr/share/glib-2.0/schemas/"
    cp "../resources/org.anek.AnekEditor.gschema.xml" "$pkgdir/usr/share/glib-2.0/schemas/"
    mkdir -p "$pkgdir/usr/share/applications/"
    cp "../anek.desktop" "$pkgdir/usr/share/applications/anek.desktop"
    cp "../anek-editor.desktop" "$pkgdir/usr/share/applications/anek-editor.desktop"
}
