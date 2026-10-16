# rdl-shell

<div align="center">

<p align="center">
  <a href="../../README.md">🇺🇸 English</a> · <a href="./ko.md">🇰🇷 한국어</a> · <a href="./zh-CN.md">🇨🇳 中文</a> · <a href="./es.md">🇪🇸 Español</a> · <a href="./hi.md">🇮🇳 हिन्दी</a><br />
  <a href="./ar.md">🇸🇦 العربية</a> · <a href="./pt-BR.md">🇧🇷 Português</a> · <strong>🇷🇺 Русский</strong> · <a href="./fr.md">🇫🇷 Français</a> · <a href="./id.md">🇮🇩 Bahasa Indonesia</a>
</p>

> Расширение оболочки Проводника Windows для [remote-dl](https://github.com/GHSFS/remote-dl) — добавляет пункт "Send to remote-dl" в контекстное меню файлов `.url` (интернет-ярлыков).

[![Build](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml)
[![Tests](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../../LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20x64-blue)](#установка)

</div>

---

### Обзор

`rdl-shell.dll` — обработчик контекстного меню Windows Shell, который интегрирует CLI-клиент [`rdl`](https://github.com/GHSFS/remote-dl) в Проводник (Explorer). После регистрации правый клик по файлу интернет-ярлыка `.url` показывает новый пункт меню — **Send to remote-dl** (при выборе нескольких файлов: **Send N shortcuts to remote-dl**) — который читает URL из ярлыка и ставит её в очередь на настроенном edge-worker.

#### ✨ Ключевые возможности (v0.2.0)
- **Пакетная обработка нескольких файлов**: выбор нескольких `.url` ярлыков одновременно; постановка всех ярлыков в очередь в один клик.
- **Полное отсутствие зависаний Проводника**: неблокирующий асинхронный рабочий поток с фиксацией времени жизни через RAII `DllLockGuard`.
- **Универсальная поддержка кодировок**: автоматический парсинг UTF-8, UTF-16LE (с BOM и без него) и UTF-16BE с регистронезависимым поиском ключа `url=`.
- **Строгая валидация URL**: предварительная проверка допустимости только схем `http://` и `https://`.
- **Пул сетевых соединений**: кэширование клиента `reqwest` (`OnceLock`) с повторным использованием TCP Keep-Alive и DNS-кэша.

Это третий инструмент персональной экосистемы `remote-dl`:

| Инструмент | Тип | Репозиторий |
|---|---|---|
| `rdl.exe` | терминальный CLI | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | системный трей + слежение за буфером обмена | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **контекстное меню Explorer** | этот репозиторий |

Все три инструмента используют один и тот же файл конфигурации, написанный CLI (`%APPDATA%\rdl\config.json`).

### Архитектура

DLL реализует четыре стандартных COM-входа и два Shell-интерфейса:

| Символ | Назначение |
|---|---|
| `DllGetClassObject` | Возвращает `IClassFactory` для нашего CLSID |
| `DllCanUnloadNow` | Сообщает, удерживают ли DLL внешние ссылки |
| `DllRegisterServer` | Создаёт записи реестра (вызывается `regsvr32`) |
| `DllUnregisterServer` | Удаляет записи (`regsvr32 /u`) |
| `IShellExtInit` | Получает выделение файлов от Explorer |
| `IContextMenu` | Рендерит пункт меню и обрабатывает `InvokeCommand` |

### Структура проекта

```
rdl-shell/
├── Cargo.toml             Манифест пакета, crate-type=cdylib (вывод DLL)
├── rust-toolchain.toml    Закрепление toolchain (stable / x86_64-pc-windows-msvc)
├── .cargo/config.toml     Цель сборки + статическая MSVC CRT
├── build.rs               Компиляция ресурсов + добавление /DEF: к компоновщику
├── src/
│   ├── lib.rs             CLSID, REF_COUNT, DLL_HMODULE, 4 точки входа COM
│   ├── factory.rs         Реализация IClassFactory
│   ├── handler.rs         Реализация IShellExtInit + IContextMenu
│   ├── registry.rs        DllRegisterServer / DllUnregisterServer
│   ├── client.rs          Клиент reqwest::blocking, POST /api/dl
│   ├── config.rs          Чтение %APPDATA%\rdl\config.json + DPAPI расшифровка
│   └── error.rs           Тип ошибки уровня крейта
├── tests/smoke.rs         Smoke-тест проверки сборки
├── resources/
│   ├── app.rc             Win32 resource script (manifest + VERSIONINFO)
│   ├── app.manifest       Декларация совместимости с ОС
│   └── rdl-shell.def      Module-definition: имена COM-экспортов
└── .github/workflows/
    ├── build.yml          CI cargo build + проверка dumpbin /EXPORTS
    └── test.yml           CI fmt / clippy / test
```

### Установка

> Права администратора требуются только для шага регистрации в реестре.

1. Загрузите `rdl-shell.dll` со страницы [Releases](https://github.com/GHSFS/rdl-shell/releases).
2. Поместите его в постоянный каталог (например, `C:\Tools\rdl-shell.dll`).
3. Запустите PowerShell от имени администратора и выполните:

   ```powershell
   regsvr32 C:\Tools\rdl-shell.dll
   ```

4. Перезапустите Explorer, чтобы загрузить расширение.

### Удаление

```powershell
regsvr32 /u C:\Tools\rdl-shell.dll
```

### Лицензия

MIT. См. [LICENSE](../../LICENSE).
