# rdl-shell

<div align="center">

<p align="center">
  <a href="../../README.md">🇺🇸 English</a> · <a href="./ko.md">🇰🇷 한국어</a> · <a href="./zh-CN.md">🇨🇳 中文</a> · <a href="./es.md">🇪🇸 Español</a> · <a href="./hi.md">🇮🇳 हिन्दी</a><br />
  <a href="./ar.md">🇸🇦 العربية</a> · <strong>🇧🇷 Português</strong> · <a href="./ru.md">🇷🇺 Русский</a> · <a href="./fr.md">🇫🇷 Français</a> · <a href="./id.md">🇮🇩 Bahasa Indonesia</a>
</p>

> Extensão de shell do Windows Explorer para [remote-dl](https://github.com/GHSFS/remote-dl) — adiciona a opção "Send to remote-dl" ao menu de contexto de arquivos `.url` (atalhos da Internet).

[![Build](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml)
[![Tests](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../../LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20x64-blue)](#instalação)

</div>

---

### Visão geral

`rdl-shell.dll` é um manipulador de menu de contexto do Windows Shell que integra o cliente CLI [`rdl`](https://github.com/GHSFS/remote-dl) no Explorer. Após o registro, clicar com o botão direito em um arquivo de atalho da Internet `.url` mostra uma nova entrada de menu — **Send to remote-dl** (ou **Send N shortcuts to remote-dl** ao selecionar múltiplos arquivos) — que lê a URL do atalho e a coloca na fila do edge worker configurado.

#### ✨ Principais recursos (v0.2.0)
- **Seleção de múltiplos arquivos**: Selecione vários atalhos `.url` de uma só vez e enfileire todos com apenas um clique.
- **Sem travamento do Explorer (Non-Blocking Async)**: Thread de trabalho assíncrona em segundo plano com proteção de tempo de vida RAII `DllLockGuard`.
- **Codificação universal de atalhos**: Suporte automático para decodificação de UTF-8, UTF-16LE (com ou sem BOM), UTF-16BE e detecção insensível a maiúsculas da chave `url=`.
- **Validação estrita de URL**: Garante que apenas os esquemas `http://` e `https://` sejam aceitos antes do disparo.
- **Otimização de Pool de Conexões**: Cliente `reqwest` em cache (`OnceLock`) com reaproveitamento de Keep-Alive e cache de DNS.

Esta é a terceira ferramenta do ecossistema pessoal `remote-dl`:

| Ferramenta | Formato | Repositório |
|---|---|---|
| `rdl.exe` | CLI de terminal | [`remote-dl`](https://github.com/GHSFS/remote-dl) |
| `rdl-tray.exe` | bandeja do sistema + monitor da área de transferência | [`rdl-tray`](https://github.com/GHSFS/rdl-tray) |
| **`rdl-shell.dll`** | **menu de clique direito do Explorer** | este repositório |

Todas as três ferramentas compartilham o mesmo arquivo de configuração em disco escrito pela CLI (`%APPDATA%\rdl\config.json`).

### Arquitetura

O DLL implementa os quatro pontos de entrada COM padrão e duas interfaces de Shell:

| Símbolo | Propósito |
|---|---|
| `DllGetClassObject` | Retorna o `IClassFactory` para nosso CLSID |
| `DllCanUnloadNow` | Informa se há referências externas mantendo o DLL |
| `DllRegisterServer` | Instala as entradas do registro (chamado por `regsvr32`) |
| `DllUnregisterServer` | Remove as entradas (chamado por `regsvr32 /u`) |
| `IShellExtInit` | Recebe a seleção de arquivos do Explorer |
| `IContextMenu` | Desenha a entrada do menu e responde a `InvokeCommand` |

### Estrutura do projeto

```
rdl-shell/
├── Cargo.toml             Manifesto do pacote, crate-type=cdylib (saída DLL)
├── rust-toolchain.toml    Fixação do toolchain (stable / x86_64-pc-windows-msvc)
├── .cargo/config.toml     Alvo de build + CRT estática do MSVC
├── build.rs               Compila recursos + adiciona /DEF: ao linker
├── src/
│   ├── lib.rs             CLSID, REF_COUNT, DLL_HMODULE, 4 entry points COM
│   ├── factory.rs         Implementação de IClassFactory
│   ├── handler.rs         Implementação de IShellExtInit + IContextMenu
│   ├── registry.rs        DllRegisterServer / DllUnregisterServer
│   ├── client.rs          Cliente reqwest::blocking, POST /api/dl
│   ├── config.rs          Leitura de %APPDATA%\rdl\config.json + DPAPI
│   └── error.rs           Tipo de erro do crate
├── tests/smoke.rs         Smoke test de verificação de build
├── resources/
│   ├── app.rc             Script de recursos Win32 (manifesto + VERSIONINFO)
│   ├── app.manifest       Declaração de compatibilidade de OS
│   └── rdl-shell.def      Module-definition: exports COM por nome
└── .github/workflows/
    ├── build.yml          CI cargo build + verificação dumpbin /EXPORTS
    └── test.yml           CI fmt / clippy / test
```

### Instalação

> Requer privilégios de administrador apenas para o registro.

1. Baixe o `rdl-shell.dll` na página de [Releases](https://github.com/GHSFS/rdl-shell/releases).
2. Salve-o em um local permanente (ex: `C:\Tools\rdl-shell.dll`).
3. Em um PowerShell como Administrador, execute:

   ```powershell
   regsvr32 C:\Tools\rdl-shell.dll
   ```

4. Reinicie o Explorer para recarregar as extensões de menu.

### Remoção

```powershell
regsvr32 /u C:\Tools\rdl-shell.dll
```

### Licença

MIT. Consulte [LICENSE](../../LICENSE).
