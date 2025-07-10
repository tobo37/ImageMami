# Pre-release for testers

This project provides early access builds through GitHub pre-releases. These versions are meant for testing only and may be unstable.

## How to create a pre-release

1. Run the **Pre Release** workflow on GitHub.
2. Provide a tag name (for example `v0.1.0-alpha.1`), title and optional release notes.
3. The workflow will publish a GitHub release marked as a pre-release.
4. Trigger the **Build Tauri** workflow with the same tag to build the Windows installer as well as Linux AppImage, Flatpak, and RPM packages.

## How to download

After the build completes, open the **Releases** page on GitHub and download the generated assets. You will find an installer for Windows along with Linux AppImage, Flatpak, and RPM packages.
