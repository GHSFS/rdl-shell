# rdl-shell

<div align="center">

<p align="center">
  <a href="../../README.md">🇺🇸 English</a> · <a href="./ko.md">🇰🇷 한국어</a> · <a href="./zh-CN.md">🇨🇳 中文</a> · <a href="./es.md">🇪🇸 Español</a> · <a href="./hi.md">🇮🇳 हिन्दी</a><br />
  <a href="./ar.md">🇸🇦 العربية</a> · <a href="./pt-BR.md">🇧🇷 Português</a> · <a href="./ru.md">🇷🇺 Русский</a> · <strong>🇫🇷 Français</strong> · <a href="./id.md">🇮🇩 Bahasa Indonesia</a>
</p>

> Extension de menu contextuel Windows Explorer pour [remote-dl](https://github.com/GHSFS/remote-dl) — ajoute une option "Send to remote-dl" au menu clic droit des fichiers raccourcis `.url`.

[![Build](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml)
[![Tests](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../../LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20x64-blue)](#installation)

</div>

---

### Aperçu

`rdl-shell.dll` est un gestionnaire de menu contextuel Windows Shell qui intègre le client CLI [`rdl`](https://github.com/GHSFS/remote-dl) dans Explorer. Une fois enregistré, un clic droit sur un fichier raccourci Internet `.url` affiche une nouvelle entrée — **Send to remote-dl** (ou **Send N shortcuts to remote-dl** en cas de sélection multiple) — qui extrait l'URL du fichier et la transmet au worker configuré.

#### ✨ Fonctionnalités clés (v0.2.0)
- **Sélection multi-fichiers** : Sélectionnez plusieurs fichiers `.url` simultanément ; met en file d'attente tous les raccourcis en un seul clic.
- **Aucun gel d'Explorer (Non-Blocking Async)** : Déporté sur un thread d'arrière-plan avec verrou de cycle de vie RAII `DllLockGuard` pour un retour UI immédiat.
- **Décodage d'encodage universel** : Détection et décodage automatiques pour UTF-8, UTF-16LE (avec ou sans BOM), UTF-16BE et clé `url=` insensible à la casse.
- **Validation stricte des URL** : Vérifie et autorise uniquement les protocoles `http://` et `https://` avant toute transmission réseau.
- **Pool de connexions HTTP** : Client `reqwest` optimisé en cache (`OnceLock`) pour réutiliser les connexions TCP Keep-Alive et le cache DNS.

Troisième composant de l'écosystème personnel `remote-dl` :

| Outil | Interface | Dépôt |
|---|---|---|
| `rdl.exe` | CLI terminal | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | zone de notification + presse-papiers | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **menu contextuel Explorer** | ce dépôt |

Tous les trois partagent la même configuration sur disque générée par le CLI (`%APPDATA%\rdl\config.json`).

### Installation

> Nécessite des privilèges administrateur uniquement pour l'enregistrement.

1. Téléchargez `rdl-shell.dll` depuis la page des [Releases](https://github.com/GHSFS/rdl-shell/releases).
2. Placez-le dans un répertoire stable (ex: `C:\Tools\rdl-shell.dll`).
3. Ouvrez un terminal PowerShell en tant qu'administrateur et exécutez :

   ```powershell
   regsvr32 C:\Tools\rdl-shell.dll
   ```

4. Redémarrez Explorer pour recharger les extensions.

### Désinstallation

```powershell
regsvr32 /u C:\Tools\rdl-shell.dll
```

### Licence

MIT. Voir [LICENSE](../../LICENSE).
