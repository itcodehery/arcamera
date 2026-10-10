# Maintainer: Your Name <you@example.com>
pkgname=arcamera
pkgver=0.1.0
pkgrel=1
pkgdesc="A high-performance Tauri-based camera application"
arch=('x86_64')
url="https://github.com/youruser/arcamera"
license=('GPL-3.0-or-later') # Update if a different license is used
depends=('webkit2gtk-4.1' 'gtk3' 'cairo' 'gdk-pixbuf2' 'glib2' 'pango')
makedepends=('cargo' 'nodejs' 'npm')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz"
        "$pkgname.desktop")
sha256sums=('SKIP'
            'SKIP')

build() {
    cd "$pkgname-$pkgver"
    npm install
    # Build frontend
    npm run build
    
    # Build Rust backend
    cd src-tauri
    cargo build --release --locked
}

package() {
    cd "$pkgname-$pkgver"
    
    # Install binaries to a dedicated lib folder since it requires the external ffmpeg binary next to it
    install -Dm755 "src-tauri/target/release/$pkgname" -t "$pkgdir/usr/lib/$pkgname/"
    install -Dm755 "src-tauri/bin/ffmpeg-x86_64-unknown-linux-gnu" -t "$pkgdir/usr/lib/$pkgname/"
    
    # Create a symlink in /usr/bin
    mkdir -p "$pkgdir/usr/bin"
    ln -sf "/usr/lib/$pkgname/$pkgname" "$pkgdir/usr/bin/$pkgname"
    
    # Install icon and desktop file
    install -Dm644 "src-tauri/icons/icon.png" "$pkgdir/usr/share/icons/hicolor/512x512/apps/$pkgname.png"
    install -Dm644 "../$pkgname.desktop" -t "$pkgdir/usr/share/applications/"
    
    # Install license if it exists
    if [ -f "LICENSE" ]; then
        install -Dm644 LICENSE -t "$pkgdir/usr/share/licenses/$pkgname/"
    fi
}
