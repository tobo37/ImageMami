import { invoke } from "@tauri-apps/api/core";

export interface ExternalDevice {
  name: string;
  path: string;
  total: number;
}

export interface DuplicateGroup {
  tag: string;
  hash: string;
  paths: string[];
}

export interface BlackholeFolder {
  path: string;
  files: string[];
}

export async function listExternalDevices(): Promise<ExternalDevice[]> {
  return invoke<ExternalDevice[]>("list_external_devices");
}

export async function listAllDisks(): Promise<ExternalDevice[]> {
  return invoke<ExternalDevice[]>("list_all_disks");
}

export async function importDevice(
  devicePath: string,
  destPath: string,
): Promise<void> {
  return invoke("import_device", { devicePath, destPath });
}

export async function scanFolder(
  path: string,
  tags: string[],
): Promise<DuplicateGroup[]> {
  return invoke<DuplicateGroup[]>("scan_folder_multi_stream", { path, tags });
}

export async function recordDecision(
  tag: string,
  path: string,
  del: boolean | null,
): Promise<void> {
  return invoke("record_decision", { tag, path, delete: del });
}

export async function deleteFiles(paths: string[]): Promise<void> {
  return invoke("delete_files", { paths });
}

export async function cancelScan(): Promise<void> {
  return invoke("cancel_scan");
}

export async function findImages(path: string): Promise<string[]> {
  return invoke<string[]>("find_images", { path });
}

export async function sortImages(path: string): Promise<void> {
  return invoke("sort_images", { path });
}

export async function generateThumbnail(
  path: string,
  maxSize?: number,
): Promise<string> {
  return invoke<string>("generate_thumbnail", { path, maxSize });
}

export async function scanBlackhole(
  rootPath: string,
  destPath: string,
): Promise<BlackholeFolder[]> {
  return invoke<BlackholeFolder[]>("scan_blackhole_stream", {
    rootPath,
    destPath,
  });
}

export async function importBlackhole(
  files: string[],
  destPath: string,
  cut: boolean,
): Promise<void> {
  return invoke("import_blackhole", { files, destPath, cut });
}

export async function exportTraining(path: string): Promise<void> {
  return invoke("export_training", { path });
}
