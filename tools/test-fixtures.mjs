// Shared temp-fixture cleanup that survives Windows fixture content.
//
// Node's rmSync({ recursive: true, force: true }) fails on two fixture
// byproducts under Windows:
//   - Git marks loose objects read-only (mode 444) -> rmSync EPERM.
//   - Junctions whose target was deleted first -> rmSync fails with
//     ERROR_CANT_RESOLVE_FILENAME ("The name of the file cannot be resolved").
// lstat never resolves links, so symlinks and junctions unlink directly;
// chmod clears the read-only bit before unlinking files.

import { chmodSync, lstatSync, readdirSync, rmdirSync, unlinkSync } from "node:fs";
import { join } from "node:path";

export function removeTree(path) {
  let stat;
  try {
    stat = lstatSync(path);
  } catch {
    return; // already gone
  }
  if (stat.isSymbolicLink() || !stat.isDirectory()) {
    if (process.platform === "win32") {
      try { chmodSync(path, 0o666); } catch { /* best effort */ }
    }
    unlinkSync(path);
    return;
  }
  for (const entry of readdirSync(path)) removeTree(join(path, entry));
  rmdirSync(path);
}
