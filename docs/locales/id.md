# rdl-shell

<div align="center">

<p align="center">
  <a href="../../README.md">🇺🇸 English</a> · <a href="./ko.md">🇰🇷 한국어</a> · <a href="./zh-CN.md">🇨🇳 中文</a> · <a href="./es.md">🇪🇸 Español</a> · <a href="./hi.md">🇮🇳 हिन्दी</a><br />
  <a href="./ar.md">🇸🇦 العربية</a> · <a href="./pt-BR.md">🇧🇷 Português</a> · <a href="./ru.md">🇷🇺 Русский</a> · <a href="./fr.md">🇫🇷 Français</a> · <strong>🇮🇩 Bahasa Indonesia</strong>
</p>

> Ekstensi menu konteks Windows Explorer untuk [remote-dl](https://github.com/GHSFS/remote-dl) — menambahkan entri "Send to remote-dl" pada menu klik kanan file pintasan `.url` (Internet Shortcut).

[![Build](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml)
[![Tests](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../../LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20x64-blue)](#instalasi)

</div>

---

### Ikhtisar

`rdl-shell.dll` adalah handler menu konteks Windows Shell yang mengintegrasikan klien CLI [`rdl`](https://github.com/GHSFS/remote-dl) langsung ke dalam Explorer. Setelah terdaftar, klik kanan pada file Internet Shortcut `.url` akan menampilkan opsi menu baru — **Send to remote-dl** (atau **Send N shortcuts to remote-dl** saat memilih beberapa file) — yang membaca URL dari pintasan dan memasukkannya ke antrean worker yang telah dikonfigurasi.

#### ✨ Fitur Utama (v0.2.0)
- **Pemilihan Banyak File Sekaligus**: Pilih beberapa pintasan `.url` bersamaan; memasukkan semua tautan ke antrean dalam satu klik.
- **Bebas Hang Explorer (Non-Blocking Async)**: Pemrosesan dilakukan pada thread worker latar belakang dengan penjaga waktu hidup RAII `DllLockGuard` agar antarmuka Explorer langsung responsif.
- **Dukungan Encoding Pintasan Universal**: Deteksi dan decoding otomatis untuk UTF-8, UTF-16LE (dengan/tanpa BOM), UTF-16BE, dan kunci `url=` tanpa membedakan huruf besar/kecil.
- **Validasi URL Ketat**: Memeriksa dan memastikan hanya skema `http://` dan `https://` yang diizinkan sebelum pengiriman.
- **Optimalisasi Pool Koneksi**: Klien `reqwest` yang di-cache (`OnceLock`) dengan Keep-Alive dan cache DNS untuk eksekusi berkecepatan tinggi.

Bagian ketiga dari ekosistem pribadi `remote-dl`:

| Alat | Antarmuka | Repositori |
|---|---|---|
| `rdl.exe` | CLI terminal | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | baki sistem + pemantau papan klip | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **menu klik kanan Explorer** | repositori ini |

Ketiganya berbagi file konfigurasi disk yang sama yang ditulis oleh CLI (`%APPDATA%\rdl\config.json`).

### Instalasi

> Memerlukan hak administrator hanya untuk langkah pendaftaran.

1. Unduh `rdl-shell.dll` dari halaman [Releases](https://github.com/GHSFS/rdl-shell/releases).
2. Simpan di direktori permanen (contoh: `C:\Tools\rdl-shell.dll`).
3. Buka PowerShell sebagai Administrator dan jalankan:

   ```powershell
   regsvr32 C:\Tools\rdl-shell.dll
   ```

4. Muat ulang Explorer untuk menerapkan menu baru.

### Pencopotan

```powershell
regsvr32 /u C:\Tools\rdl-shell.dll
```

### Lisensi

MIT. Lihat [LICENSE](../../LICENSE).
