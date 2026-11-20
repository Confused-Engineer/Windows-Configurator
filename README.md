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

The UI renders with **wgpu on DirectX 12** by default, which falls back to the
built-in *Microsoft Basic Render Driver* (WARP) on VMs without 3D acceleration
and in RDP sessions — where the old OpenGL-only build showed no window. If
that fails it retries with OpenGL. To force a renderer:

```
Windows_Configurator.exe --renderer software   # DX12 WARP (CPU)
Windows_Configurator.exe --renderer glow       # OpenGL
Windows_Configurator.exe --renderer wgpu       # GPU via DX12/Vulkan
```

(or set `WINCONFIG_RENDERER`). The renderer in use is shown under *Settings*.

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
