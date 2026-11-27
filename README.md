# Windows Configurator

A portable toolbox for setting up, troubleshooting and maintaining Windows PCs.
It is a single `.exe` meant to live on a USB drive next to its `config.ini`.

## Features

- **Applications** – install favourites from your config or search the whole
  winget catalog; install with progress or silently in the background; upgrade
  everything at once; per-app install options (all-users scope, installer
  type, architecture, extra winget arguments); run your own installers, scripts and download links.
- **Install winget** – on debloated / Store-less Windows, installs winget from
  the PowerShell Gallery (NuGet → `Microsoft.WinGet.Client` →
  `Repair-WinGetPackageManager -AllUsers`). Offered automatically when winget
  is missing, and under *Settings* as "Repair / update winget".
- **Tokens** – keep license keys / enrolment tokens in the config and copy
  them with one click.
- **Launch** – common Windows Settings pages, Control Panel applets and power
  commands.
- **Troubleshooting** – ping, pathping, tracert, nslookup, DNS flush, IP
  release/renew, Winsock reset, firewall toggles, SFC, DISM, gpupdate/gpresult.
- **System information** – `systeminfo`, `ipconfig /all`, `netstat`, `arp`,
  route table; refreshed only while the page is open.
- **Self-update** to the latest nightly build.

## Portable by design

- `config.ini` is always read from the folder containing the exe (not the
  working directory). Edits are picked up automatically. If it is missing,
  *Settings → Create default config.ini* writes a commented template.
- Nothing is written to `%APPDATA%`. Downloaded installers go to
  `%TEMP%\WindowsConfigurator` so the USB drive is not filled up.
- The VC runtime is linked statically; no redistributables are needed.

## Virtual machines / RDP

The UI is drawn entirely on the CPU (egui + a tile-caching software
rasterizer, presented through GDI via `softbuffer`), with no Direct3D or
OpenGL. It therefore renders the same in GPU-less VMs, RDP sessions and on
real hardware: the window opens immediately, clicks stay fast, and no
graphics driver is involved. (Older builds used OpenGL only, which is why
the window never appeared in VMs.)

## Config file

See [`assets/config.ini`](assets/config.ini) for the documented template:

| Section | Entry | Action |
|---|---|---|
| `[winget-<Category>]` | `Name = Winget.Id [winget args]` | Favourite apps, grouped by category; optional args e.g. `--scope machine --installer-type wix` |
| `[Programs]` | `Name = path\to\file.exe` | Opened directly (paths relative to the exe) |
| `[Powershell]` | `Name = script.ps1` | Run with `-ExecutionPolicy Bypass` |
| `[<EXT>-Online]` | `Name = https://…` | Downloaded as `Name.<ext>` and opened |
| `[Tokens]` | `Name = value` | Shown with a copy button |

## Building

```
cargo build --release
cargo test                         # unit tests
cargo test -- --ignored            # + live winget catalog test
```
