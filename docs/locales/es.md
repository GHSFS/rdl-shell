# rdl-shell

<div align="center">

<p align="center">
  <a href="../../README.md">🇺🇸 English</a> · <a href="./ko.md">🇰🇷 한국어</a> · <a href="./zh-CN.md">🇨🇳 中文</a> · <strong>🇪🇸 Español</strong> · <a href="./hi.md">🇮🇳 हिन्दी</a><br />
  <a href="./ar.md">🇸🇦 العربية</a> · <a href="./pt-BR.md">🇧🇷 Português</a> · <a href="./ru.md">🇷🇺 Русский</a> · <a href="./fr.md">🇫🇷 Français</a> · <a href="./id.md">🇮🇩 Bahasa Indonesia</a>
</p>

> Extensión de shell de Windows Explorer para [remote-dl](https://github.com/GHSFS/remote-dl): añade una entrada "Send to remote-dl" al menú contextual de archivos `.url` (accesos directos de Internet).

[![Build](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml)
[![Tests](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../../LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20x64-blue)](#instalación)

</div>

---

### Descripción general

`rdl-shell.dll` es un manejador de menú contextual de Windows Shell que integra el cliente CLI [`rdl`](https://github.com/GHSFS/remote-dl) en Explorer. Una vez registrado, al hacer clic derecho sobre un archivo de acceso directo de Internet `.url` aparece una nueva entrada de menú — **Send to remote-dl** (o **Send N shortcuts to remote-dl** si se seleccionan varios archivos) — que lee la URL del acceso directo y la encola rápidamente en el edge worker configurado.

#### ✨ Características principales (v0.2.0)
- **Selección de múltiples archivos**: Seleccione varios archivos `.url` a la vez y encole todos los accesos directos con un solo clic.
- **Cero bloqueos de Explorer (Non-Blocking Async)**: Hilo de trabajo asíncrono en segundo plano protegido con `DllLockGuard` RAII para evitar el congelamiento de la interfaz.
- **Compatibilidad universal de codificación**: Análisis automático para UTF-8, UTF-16LE (con o sin BOM), UTF-16BE y búsqueda de clave `url=` insensible a mayúsculas/minúsculas.
- **Validación estrita de esquemas URL**: Comprueba y restringe estrictamente los esquemas a `http://` y `https://` antes del envío.
- **Optimización de Connection Pooling**: Cliente `reqwest` en caché (`OnceLock`) para reutilización de conexiones Keep-Alive y resolución DNS.

Esta es la tercera herramienta del ecosistema personal `remote-dl`:

| Herramienta | Forma | Repositorio |
|---|---|---|
| `rdl.exe` | CLI de terminal | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | bandeja del sistema + vigilancia del portapapeles | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **menú contextual de Explorer** | este repositorio |

Las tres herramientas comparten el mismo archivo de configuración en disco escrito por la CLI (`%APPDATA%\rdl\config.json`).

### Arquitectura

El DLL implementa los cuatro puntos de entrada COM estándar y dos interfaces de Shell:

| Símbolo | Propósito |
|---|---|
| `DllGetClassObject` | Devuelve el `IClassFactory` para nuestro CLSID |
| `DllCanUnloadNow` | Informa si quedan referencias que retengan el DLL |
| `DllRegisterServer` | Instala entradas del registro (llamado por `regsvr32`) |
| `DllUnregisterServer` | Elimina las entradas (llamado por `regsvr32 /u`) |
| `IShellExtInit` | Recibe la selección de archivos desde Explorer |
| `IContextMenu` | Renderiza la entrada del menú y reacciona a `InvokeCommand` |

### Estructura del proyecto

```
rdl-shell/
├── Cargo.toml             Manifiesto del paquete, crate-type=cdylib (salida DLL)
├── rust-toolchain.toml    Fijación del toolchain (stable / x86_64-pc-windows-msvc)
├── .cargo/config.toml     Destino de build + CRT estática de MSVC
├── build.rs               Compila recursos + añade /DEF: al linker
├── src/
│   ├── lib.rs             CLSID, REF_COUNT, DLL_HMODULE, 4 puntos de entrada COM
│   ├── factory.rs         Implementación de IClassFactory
│   ├── handler.rs         Implementación de IShellExtInit + IContextMenu
│   ├── registry.rs        DllRegisterServer / DllUnregisterServer
│   ├── client.rs          Cliente reqwest::blocking, POST /api/dl
│   ├── config.rs          Lectura de %APPDATA%\rdl\config.json + descifrado DPAPI
│   └── error.rs           Tipo de error del crate
├── tests/smoke.rs         Smoke test de verificación de build
├── resources/
│   ├── app.rc             Script de recursos Win32 (manifiesto + VERSIONINFO)
│   ├── app.manifest       Declaración de compatibilidad de OS
│   └── rdl-shell.def      Module-definition: exports COM por nombre
└── .github/workflows/
    ├── build.yml          CI cargo build + verificación dumpbin /EXPORTS
    └── test.yml           CI fmt / clippy / test
```

### Compatibilidad

| Eje | Soportado |
|---|---|
| Sistema operativo | Windows 10 1809+ y Windows 11 |
| Arquitectura | Solo x86_64 (coincide con el proceso de Explorer) |
| Toolchain de Rust | 1.75+ (`rust-toolchain.toml` fija stable) |
| Enlazador | MSVC (Visual Studio Build Tools 2022) |
| Registro | Por máquina (`HKCR`) — `regsvr32` requiere elevación |

El DLL se enlaza estáticamente con el runtime de MSVC, por lo que no tiene dependencias adicionales aparte de las librerías nativas del sistema Windows (`KERNEL32.DLL`, `USER32.DLL`, `SHELL32.DLL`, `ADVAPI32.DLL`, `OLE32.DLL`, `CRYPT32.DLL`).

### Seguridad

- **Token cifrado en reposo**: El token bearer se cifra con Windows DPAPI antes de guardarse en disco.
- **Canal seguro**: `reqwest` se compila con TLS forzado y el token solo viaja en encabezados `Authorization`.
- **Mínimo privilegio**: Solo la registración inicial con `regsvr32` necesita administrador; en tiempo de ejecución opera con privilegios estándar de Explorer.
- **Sin telemetría**: No realiza llamadas externas salvo al worker configurado.
- **Desinstalación idempotente**: Ejecutar `regsvr32 /u` repetidamente es seguro.

### Instalación

> Requiere permisos de administrador solo para el registro.

1. Descargue `rdl-shell.dll` desde la página de [Releases](https://github.com/GHSFS/rdl-shell/releases).
2. Ubíquelo en una ruta fija (por ejemplo, `C:\Tools\rdl-shell.dll`).
3. Abra PowerShell como Administrador y ejecute:

   ```powershell
   regsvr32 C:\Tools\rdl-shell.dll
   ```

4. Reinicie Explorer para cargar la extensión.

### Desinstalación

```powershell
regsvr32 /u C:\Tools\rdl-shell.dll
```

### Licencia

MIT. Consulte [LICENSE](../../LICENSE).
