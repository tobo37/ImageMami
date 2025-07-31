# ImageMami

## For end users

ImageMami helps you import, organize and deduplicate your photos on Linux. The project started as a hobby replacement for Lightroom's import when I switched to Linux. To explore the limits of AI, roughly two thirds of the code base was machine generated.

- Import images from folders or removable media
- Sort photos into folders
- Detect duplicates using exact or perceptual hashing

![Screenshot placeholder](docs/images/screenshot-main.png)

## For developers

Built with **Tauri**, **Vue 3**, and **TypeScript**. Large parts of the project were generated with AI to experiment with new workflows.

### Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

### Type Support For `.vue` Imports in TS

Since TypeScript cannot handle type information for `.vue` imports, they are shimmed to be a generic Vue component type by default. In most cases this is fine if you don't really care about component prop types outside of templates. However, if you wish to get actual prop types in `.vue` imports (for example to get props validation when using manual `h(...)` calls), you can enable Volar's Take Over mode by following these steps:

1. Run `Extensions: Show Built-in Extensions` from VS Code's command palette, look for `TypeScript and JavaScript Language Features`, then right click and select `Disable (Workspace)`. By default, Take Over mode will enable itself if the default TypeScript extension is disabled.
2. Reload the VS Code window by running `Developer: Reload Window` from the command palette.

You can learn more about Take Over mode [here](https://github.com/johnsoncodehk/volar/discussions/471).

### Duplicate detection modes

The duplicate scanner can run in either **Exact** mode using content hashes or in a **Perceptual** mode using dHash. Choose the desired mode in the Duplicate view before starting a scan.

### Pre-release builds

Early testers can try unstable builds from GitHub pre-releases. See [docs/PRE_RELEASE.md](docs/PRE_RELEASE.md) for instructions on creating and downloading pre-releases.

### Install for Ubuntu

```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
npm install
npm run tauri build
```

### Flamegraph

```bash
cargo install flamegraph
npm run tauri build
flamegraph --root -- ./src-tauri/target/release/imagemami
# or run without root privileges
flamegraph -- ./src-tauri/target/release/imagemami
```

![Developer screenshot placeholder](docs/images/screenshot-dev.png)
