# Maintainer: sharkthakftw <sharkthakftw@gmail.com>
pkgname=wikid
pkgver=3.2.0
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
sha256sums=('cc3cf4f396776ed6a8c2f8729e60382f740dd1cfec23d4ce85060e7319e53057')

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
