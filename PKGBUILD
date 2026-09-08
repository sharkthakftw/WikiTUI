# Maintainer: sharkthakftw <sharkthakftw@gmail.com>
pkgname=wikid
pkgver=3.2.1
pkgrel=1
pkgdesc="feature-rich terminal wikipedia client"
arch=('x86_64' 'aarch64')
url="https://github.com/sharkthakftw/wikid"
license=('MIT')
depends=('gcc-libs' 'glibc')
optdepends=(
  'mpv: spoken article audio playback (recommended)'
  'ffmpeg: alternative audio playback backend (ffplay)'
  'vlc: alternative audio playback backend (cvlc)'
)
makedepends=('cargo')
options=('!lto')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('2d8e0dda910864fe1045e9be938a8248b650a6bd339c8785d2ebc7100be1cc13')

build() {
  cd "$pkgname-$pkgver"
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  cargo build --release
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
