#!/bin/bash
#
# Dieses Skript automatisiert die Veröffentlichung eines neuen AUR-Paket-Updates.
# Es führt das Vorbereitungsskript aus, kopiert die Ergebnisse ins Nachbar-Repo,
# committet und pusht die Änderungen automatisch.
#
# WICHTIG:
# - Muss aus dem Haupt-Projektverzeichnis (z.B. ImageMami) ausgeführt werden.
# - Das AUR-Repo muss sich im selben übergeordneten Ordner befinden.

set -euo pipefail

# --- Konfiguration ---
# Name deines Pakets und des AUR-Repo-Ordners
PKG_NAME="imagemami"
# --- Ende Konfiguration ---

# Pfade definieren
AUR_REPO_PATH="../${PKG_NAME}"
PREPARE_SCRIPT="./prepare-aur.sh"
GENERATED_FILES_DIR="./${PKG_NAME}"

### 1. Voraussetzungen prüfen
echo " Überprüfe Voraussetzungen..."
if [ ! -f "$PREPARE_SCRIPT" ]; then
    echo "❌ Fehler: Das Skript '$PREPARE_SCRIPT' wurde nicht gefunden."
    exit 1
fi
if [ ! -d "$AUR_REPO_PATH" ]; then
    echo "❌ Fehler: Das AUR-Repository '${AUR_REPO_PATH}' wurde nicht gefunden."
    exit 1
fi
echo "  ✅ Voraussetzungen erfüllt."


### 2. (Schritt 3) AUR-Dateien vorbereiten
echo -e "\n Führe das Vorbereitungsskript aus..."
bash "$PREPARE_SCRIPT"


### 3. Version für Commit-Nachricht auslesen
VERSION=$(grep '^pkgver=' "${GENERATED_FILES_DIR}/PKGBUILD" | cut -d'=' -f2)
if [ -z "$VERSION" ]; then
    echo "❌ Fehler: Konnte die Version nicht aus der PKGBUILD lesen."
    exit 1
fi
echo "  ✅ Version ${VERSION} wird veröffentlicht."


### 4. (Schritt 4) Dateien ins AUR-Repo kopieren
echo -e "\n Kopiere PKGBUILD und .SRCINFO..."
cp "${GENERATED_FILES_DIR}/PKGBUILD" "${AUR_REPO_PATH}/"
cp "${GENERATED_FILES_DIR}/.SRCINFO" "${AUR_REPO_PATH}/"
echo "  ✅ Dateien kopiert."


### 5. (Schritt 5) Änderungen committen und pushen
echo -e "\n Veröffentliche Änderungen im AUR-Repository..."
cd "$AUR_REPO_PATH"

# Prüfen, ob es überhaupt Änderungen gibt, um leere Commits zu vermeiden
if git diff-index --quiet HEAD --; then
    echo "  ℹ️ Keine Änderungen festgestellt. Das Repository ist bereits aktuell."
    echo -e "\n✅ Veröffentlichung abgeschlossen."
    exit 0
fi

git add PKGBUILD .SRCINFO

COMMIT_MSG="Update to version ${VERSION}"
git commit -m "$COMMIT_MSG"
echo "  ✅ Commit erstellt: '$COMMIT_MSG'"

git push
echo "  ✅ Erfolgreich zu AUR gepusht."

echo -e "\n🎉 Komplett fertig! Dein AUR-Paket ist aktuell."