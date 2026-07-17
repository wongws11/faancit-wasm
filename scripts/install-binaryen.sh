#!/bin/sh
set -eu

version=131
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
install_dir="$root/.tools/binaryen-version_$version"
wasm_opt="$install_dir/bin/wasm-opt"

if [ -x "$wasm_opt" ] && [ "$($wasm_opt --version)" = "wasm-opt version $version (version_$version)" ]; then
    exit 0
fi

case "$(uname -s)-$(uname -m)" in
    Darwin-arm64)
        platform=arm64-macos
        checksum=e441b48dc22163d209b4f05e44dc7210909b01237642b6c9ae48fd710a3ef83e
        ;;
    Darwin-x86_64)
        platform=x86_64-macos
        checksum=d209fadd8a894bdaf3bd3612a23c32a0af184d2f4a979b8c789e6e4f6a4de883
        ;;
    Linux-aarch64)
        platform=aarch64-linux
        checksum=ba991f677edd9a21d2bc96c0144bc8ac5b112d4d98a3eb266e075e22e557df2a
        ;;
    Linux-x86_64)
        platform=x86_64-linux
        checksum=b5bf1f0eaf17c63ee588ff7a5954dc8f6ce2c26989051c66f24dfe9ece3e46db
        ;;
    *)
        echo "Binaryen $version is not available for $(uname -s)-$(uname -m)" >&2
        exit 1
        ;;
esac

archive="binaryen-version_$version-$platform.tar.gz"
temporary=$(mktemp -d "${TMPDIR:-/tmp}/faancit-binaryen.XXXXXX")
trap 'rm -rf "$temporary"' EXIT HUP INT TERM

curl --fail --location --output "$temporary/$archive" \
    "https://github.com/WebAssembly/binaryen/releases/download/version_$version/$archive"

if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "$temporary/$archive" | cut -d ' ' -f 1)
else
    actual=$(shasum -a 256 "$temporary/$archive" | cut -d ' ' -f 1)
fi
if [ "$actual" != "$checksum" ]; then
    echo "Binaryen checksum mismatch: expected $checksum, found $actual" >&2
    exit 1
fi

tar --extract --gzip --file "$temporary/$archive" --directory "$temporary"
mkdir -p "$root/.tools"
rm -rf "$install_dir"
mv "$temporary/binaryen-version_$version" "$install_dir"
