#!/bin/sh
# Instala lo que necesita la app, y dice qué falta del sistema para crear un examen.
set -e
cd "$(dirname "$0")"
command -v rustup >/dev/null || { echo "Falta Rust: instálalo desde https://rustup.rs" >&2; exit 1; }
node -e 'process.exit(parseInt(process.versions.node) < 22 ? 1 : 0)' 2>/dev/null || { echo "Falta Node 22 o más nuevo" >&2; exit 1; }
# Instala la versión y el target de rust-toolchain.toml.
rustup toolchain install
# wasm-bindgen-cli tiene que tener la versión del crate wasm-bindgen de Cargo.lock.
version=$(grep -A1 '^name = "wasm-bindgen"$' app/Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p')
if [ "$(wasm-bindgen --version 2>/dev/null)" != "wasm-bindgen $version" ]; then
  cargo install wasm-bindgen-cli --version "$version" --locked
fi
(cd app/interfaz && npm ci)
falta=""
for h in pdfinfo pdftoppm pdftotext ffmpeg; do
  command -v "$h" >/dev/null || falta="$falta $h"
done
if [ -n "$falta" ]; then
  echo "Para crear un examen faltan:$falta. Instala poppler y ffmpeg con el gestor de paquetes de tu sistema."
fi
