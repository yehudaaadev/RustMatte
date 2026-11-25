# Test Fixtures and Repository Image Provenance

This register catalogs every file in `tests/fixtures/` and every image/icon asset tracked in this repository.

> [!IMPORTANT]
> **Owner Review Required**: None of the photographic fixtures in `tests/fixtures/` include upstream attribution URLs or license files in the repository. Before publishing or promoting a release, review every `TODO(owner)` entry below and confirm you hold commercial/redistribution rights (or CC0/public-domain provenance) and model releases for any identifiable people.

## 1. Files in `tests/fixtures/`

| File | Bytes | Dimensions | Tracked in Git | Source | License | Flag / Owner Action |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `baju1.jpg` | 26,805 | 564x832 | Yes | `TODO(owner)` | `TODO(owner)` | **Flagged**: clothing/subject photo (`564x832`); verify whether an identifiable person is visible and confirm public-domain/CC0 license. |
| `corrupt_truncated.jpg` | 300 | 564x832 (header) | Yes | `TODO(owner)` (truncated JPEG) | `TODO(owner)` | Derived truncated 300-byte JPEG; verify original source image license or replace with a self-authored truncated byte fixture. |
| `corrupt_zerobyte.jpg` | 0 | - | Yes | Synthetic empty file | CC0 / Public Domain (0 bytes) | Safe (0-byte synthetic robustness fixture). |
| `hewan1.jpg` | 135,253 | 1024x1387 | Yes | `TODO(owner)` | `TODO(owner)` | Verify photographer/stock license permits public git redistribution. |
| `hewan2.jpg` | 70,405 | 564x766 | Yes | `TODO(owner)` | `TODO(owner)` | Verify photographer/stock license permits public git redistribution. |
| `kaca1.jpg` | 260,507 | 1200x1600 | Yes | `TODO(owner)` | `TODO(owner)` | Verify photographer/stock license permits public git redistribution. |
| `large_24mp.jpg` | 2,325,581 | 6000x4000 | No (`.gitignore`) | `TODO(owner)` | `TODO(owner)` | Untracked locally; keep ignored (>1 MB) or document download source if shared with contributors. |
| `latarramai1.jpg` | 340,758 | 1200x1500 | Yes | `TODO(owner)` | `TODO(owner)` | **Flagged**: crowded scene (`latarramai`); may contain identifiable people and unknown license. |
| `latarramai2.jpg` | 385,801 | 1200x1600 | Yes | `TODO(owner)` | `TODO(owner)` | **Flagged**: crowded scene (`latarramai`); may contain identifiable people and unknown license. |
| `orang1.jpg` | 79,168 | 736x1288 | Yes | `TODO(owner)` | `TODO(owner)` | **Flagged (Person/Portrait)**: `orang` = person. Confirm whether real identifiable individual + permission/CC0 license. |
| `orang2.jpg` | 74,751 | 700x1000 | Yes | `TODO(owner)` | `TODO(owner)` | **Flagged (Person/Portrait)**: `orang` = person. Confirm whether real identifiable individual + permission/CC0 license. |
| `produk1.jpg` | 71,870 | 736x1288 | Yes | `TODO(owner)` | `TODO(owner)` | Verify photographer/stock license permits public git redistribution. |
| `produk2.jpg` | 84,805 | 736x981 | Yes | `TODO(owner)` | `TODO(owner)` | Verify photographer/stock license permits public git redistribution. |
| `rambut1.jpg` | 64,918 | 564x1128 | Yes | `TODO(owner)` | `TODO(owner)` | **Flagged (Hair/Portrait)**: `rambut` = hair portrait. Confirm whether real identifiable individual + permission/CC0 license. |
| `rambut2.jpg` | 195,999 | 1023x1537 | Yes | `TODO(owner)` | `TODO(owner)` | **Flagged (Hair/Portrait)**: `rambut` = hair portrait. Confirm whether real identifiable individual + permission/CC0 license. |
| `README.md` | 1,282 | - | Yes | Project documentation | GPL-3.0-or-later | Safe (project text). |
| `SHA256SUMS` | 554 | - | Yes | **Unexpected leftover file** | `TODO(owner)` | **Flagged**: currently contains `opencode-hop-*` hashes unrelated to `tests/fixtures/`. Replace with actual fixture SHA-256 sums or remove. |

## 2. Other Images and Icons Tracked in the Repository

| Path | Bytes | Source | License | Notes |
| :--- | :--- | :--- | :--- | :--- |
| `crates/app/src-tauri/icons/*` (16 files: `icon.png`, `icon.ico`, `icon.icns`, `32x32.png`, `128x128.png`, `128x128@2x.png`, Windows `Square*Logo.png`, `StoreLogo.png`) | 536 – 98,451 each | Default Tauri 2 scaffolding icons (`tauri-apps/tauri`) | `Apache-2.0 OR MIT` | Replace with custom RustMatte brand icons before `v1.0`. |
| `crates/app/static/favicon.png` | 1,571 | Default Tauri/Svelte template icon | `Apache-2.0 OR MIT` | Template placeholder favicon. |
| `crates/app/static/svelte.svg` | 1,950 | Svelte template logo (`sveltejs/kit`) | `MIT` | Unused template asset; safe or can be removed. |
| `crates/app/static/tauri.svg` | 2,605 | Tauri template logo (`tauri-apps/tauri`) | `Apache-2.0 OR MIT` | Unused template asset; safe or can be removed. |
| `crates/app/static/vite.svg` | 1,497 | Vite template logo (`vitejs/vite`) | `MIT` | Unused template asset; safe or can be removed. |

## 3. How to Remove Flagged Fixtures from Git History (If Needed)
Deleting a file in a new commit leaves the original JPEG bytes inside past git commits. If any fixture lacks redistribution rights or depicts an identifiable person without a model release:
1. Replace it locally with a verified CC0 / public-domain / synthetic image (e.g., from Unsplash/Pexels with documented source URL, or your own photo).
2. Rewrite git history cleanly using `git-filter-repo` (requires explicit owner decision because it changes commit hashes and requires `git push --force` on `main`):
   ```bash
   pip install git-filter-repo
   git filter-repo --invert-paths --path tests/fixtures/orang1.jpg --path tests/fixtures/orang2.jpg
   git push --force origin main
   ```
3. Note: Repository history has **not** been rewritten automatically.
