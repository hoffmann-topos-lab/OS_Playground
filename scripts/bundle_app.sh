#!/usr/bin/env bash
# Monta o bundle macOS "OS Playground.app" a partir do binário release.
# O ícone vem de assets/icons/AppIcon.icns (gerado por `make icons`).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/release/os_playground"
ICNS="$ROOT/assets/icons/AppIcon.icns"
APP="$ROOT/dist/OS Playground.app"
BUNDLE_ID="dev.osplayground.OSPlayground"
VERSION="$(awk -F'"' '/^version[[:space:]]*=/{print $2; exit}' "$ROOT/Cargo.toml")"

if [ ! -x "$BIN" ]; then
  echo "Binário release não encontrado em $BIN — rode 'make build' primeiro." >&2
  exit 1
fi

echo ">> Montando o bundle em $APP ..."
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/os_playground"

if [ -f "$ICNS" ]; then
  cp "$ICNS" "$APP/Contents/Resources/AppIcon.icns"
else
  echo "   aviso: $ICNS não existe — o app ficará com o ícone genérico." >&2
  echo "   rode 'make icons' e depois 'make app' de novo." >&2
fi

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>pt_BR</string>
	<key>CFBundleDisplayName</key>
	<string>OS Playground</string>
	<key>CFBundleExecutable</key>
	<string>os_playground</string>
	<key>CFBundleIconFile</key>
	<string>AppIcon</string>
	<key>CFBundleIdentifier</key>
	<string>$BUNDLE_ID</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>CFBundleName</key>
	<string>OS Playground</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleShortVersionString</key>
	<string>$VERSION</string>
	<key>CFBundleVersion</key>
	<string>$VERSION</string>
	<key>NSHighResolutionCapable</key>
	<true/>
</dict>
</plist>
PLIST

# Assinatura ad-hoc (não é notarização): mantém o bundle coerente para o
# Gatekeeper depois de mexermos no conteúdo. Falhar aqui não impede o app de abrir.
if ! codesign --force --sign - "$APP" >/dev/null 2>&1; then
  echo "   aviso: codesign ad-hoc falhou — o app ainda deve abrir normalmente." >&2
fi

# Faz o Finder recarregar o ícone do bundle.
touch "$APP"

echo ">> Pronto: $APP"
echo "   Abra com um duplo-clique no Finder, ou:  open '$APP'"
