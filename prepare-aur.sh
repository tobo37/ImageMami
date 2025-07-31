#!/bin/bash
#
# Dieses Skript automatisiert die Vorbereitung eines AUR-Pakets für ein Tauri-Projekt.
# Es holt das neueste Release von GitHub, lädt das .deb-Paket herunter,
# berechnet die Checksumme und generiert die PKGBUILD- und .SRCINFO-Dateien.
#
# BENÖTIGTE WERKZEUGE: curl, jq, makepkg (aus dem 'pacman-contrib' Paket)

set -euo pipefail

# --- KONFIGURATION ---
GH_REPO="tobo37/ImageMami"
PKG_NAME="imagemami" # AUR-Paketname (muss klein geschrieben sein)
# --- ENDE KONFIGURATION ---

# Überprüfen, ob die benötigten Tools installiert sind
for tool in curl jq makepkg; do
    if ! command -v "$tool" &> /dev/null; then
        echo "Fehler: Das Werkzeug '$tool' wird benötigt, ist aber nicht installiert."
        echo "Bitte installiere es, um fortzufahren (z.B. 'sudo pacman -S $tool'). 'makepkg' ist Teil von 'base-devel'."
        exit 1
    fi
done

echo " Lese neueste Release-Informationen von GitHub für '${GH_REPO}'..."
API_URL="https://api.github.com/repos/${GH_REPO}/releases/latest"
RELEASE_INFO=$(curl -s "$API_URL")

# Version extrahieren (z.B. "v1.2.3" -> "1.2.3")
PKG_VER=$(echo "$RELEASE_INFO" | jq -r '.tag_name | ltrimstr("v")')
if [ -z "$PKG_VER" ] || [ "$PKG_VER" == "null" ]; then
    echo "Fehler: Konnte die Versionsnummer nicht aus der GitHub-API-Antwort extrahieren."
    exit 1
fi
echo "  Version gefunden: ${PKG_VER}"

# Download-URL für das amd64.deb-Paket finden
DEB_URL=$(echo "$RELEASE_INFO" | jq -r '.assets[] | select(.name | endswith("_amd64.deb")) | .browser_download_url')
if [ -z "$DEB_URL" ] || [ "$DEB_URL" == "null" ]; then
    echo "Fehler: Konnte kein '_amd64.deb' Asset im neuesten Release finden."
    exit 1
fi
echo "  Download-URL gefunden: ${DEB_URL}"

# Temporäres Verzeichnis für den Download erstellen
TMP_DIR=$(mktemp -d)
trap 'rm -rf -- "$TMP_DIR"' EXIT # Stellt sicher, dass das temporäre Verzeichnis beim Beenden gelöscht wird

echo " Lade das .deb-Paket temporär herunter, um die Checksumme zu berechnen..."
curl -L -o "${TMP_DIR}/asset.deb" "$DEB_URL"

echo " Berechne SHA256-Checksumme..."
SHA256_SUM=$(sha256sum "${TMP_DIR}/asset.deb" | awk '{print $1}')
echo "  Checksumme: ${SHA256_SUM}"

# AUR-Paketverzeichnis erstellen oder leeren
AUR_DIR="${PKG_NAME}"
rm -rf "$AUR_DIR"
mkdir -p "$AUR_DIR"

echo " Generiere die PKGBUILD-Datei in '${AUR_DIR}/PKGBUILD'..."
cat << EOF > "${AUR_DIR}/PKGBUILD"
pkgname=${PKG_NAME}
pkgver=${PKG_VER}
pkgrel=1
pkgdesc="ImageMami helps you import, organize and deduplicate your photos. Tauri GUI"
arch=('x86_64')
url="https://github.com/${GH_REPO}"
license=('MIT')
depends=('webkit2gtk-4.1') # Wichtige Abhängigkeit für Tauri-Apps auf GTK-Basis
optdepends=()
provides=("\${pkgname}")
conflicts=()
source_x86_64=("\${pkgname}-\${pkgver}.deb::${DEB_URL}") # KORRIGIERTE ZEILE
sha256sums_x86_64=('${SHA256_SUM}')

package() {
    # .deb-Pakete sind 'ar'-Archive, die ein 'data.tar.xz' oder 'data.tar.gz' enthalten
    # Wir extrahieren das Datenarchiv direkt in das Paketverzeichnis
    bsdtar -x -f "\${srcdir}/data.tar."* -C "\${pkgdir}/"
}
EOF

echo " Generiere .SRCINFO aus der PKGBUILD..."
(cd "$AUR_DIR" && makepkg --printsrcinfo > .SRCINFO)

echo -e "\n✅ Fertig!"
echo "Dein AUR-Paket wurde im Verzeichnis './${AUR_DIR}' vorbereitet."
echo "Es enthält die fertigen Dateien 'PKGBUILD' und '.SRCINFO'."

trap - EXIT
rm -rf -- "$TMP_DIR"