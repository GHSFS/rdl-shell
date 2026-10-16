# rdl-shell

<div align="center">

<p align="center">
  <a href="../../README.md">🇺🇸 English</a> · <a href="./ko.md">🇰🇷 한국어</a> · <a href="./zh-CN.md">🇨🇳 中文</a> · <a href="./es.md">🇪🇸 Español</a> · <a href="./hi.md">🇮🇳 हिन्दी</a><br />
  <strong>🇸🇦 العربية</strong> · <a href="./pt-BR.md">🇧🇷 Português</a> · <a href="./ru.md">🇷🇺 Русский</a> · <a href="./fr.md">🇫🇷 Français</a> · <a href="./id.md">🇮🇩 Bahasa Indonesia</a>
</p>

> ملحق قائمة السياق لـ Windows Explorer من أجل [remote-dl](https://github.com/GHSFS/remote-dl) — يضيف خيار "Send to remote-dl" إلى قائمة النقر بزر الماوس الأيمن لملفات الاختصارات `.url`.

[![Build](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml)
[![Tests](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../../LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20x64-blue)](#التثبيت)

</div>

---

### نظرة عامة

`rdl-shell.dll` هو معالج لقائمة سياق Windows Shell يدمج عميل [`rdl`](https://github.com/GHSFS/remote-dl) في Explorer. بعد التسجيل، يؤدي النقر بزر الماوس الأيمن على ملف اختصار إنترنت `.url` إلى إظهار خيار جديد — **Send to remote-dl** (أو **Send N shortcuts to remote-dl** عند تحديد عدة ملفات) — والذي يقرأ الرابط من الاختصار ويضعه في قائمة الانتظار على الخادم المهيأ.

#### ✨ الميزات الرئيسية (v0.2.0)
- **تحديد ملفات متعددة**: اختر عدة اختصارات `.url` دفعة واحدة وأرسلها بنقرة واحدة.
- **منع تجميد Explorer (Non-Blocking Async)**: معالجة غير متزامنة في خيط منفصل مع حماية RAII `DllLockGuard` لضمان استجابة الواجهة فوراً.
- **دعم ترميز شامل**: فك ترميز تلقائي لـ UTF-8 و UTF-16LE و UTF-16BE ومفتاح `url=` غير حساس لحالة الأحرف.
- **تحقق صارم من الروابط**: قبول بروتوكولات `http://` و `https://` فقط ورفض الروابط غير الصالحة.
- **تجميع اتصالات HTTP**: إعادة استخدام اتصالات Keep-Alive وذاكرة DNS المخبأة عبر `reqwest` المؤقت (`OnceLock`).

### التثبيت

> تتطلب خطوة التسجيل صلاحيات المسؤول (Administrator).

1. قم بتنزيل `rdl-shell.dll` من صفحة [Releases](https://github.com/GHSFS/rdl-shell/releases).
2. ضعه في مسار دائم (مثل `C:\Tools\rdl-shell.dll`).
3. افتح PowerShell كمسؤول ونفّذ:

   ```powershell
   regsvr32 C:\Tools\rdl-shell.dll
   ```

4. أعد تشغيل Explorer لتطبيق التغييرات.

### إلغاء التثبيت

```powershell
regsvr32 /u C:\Tools\rdl-shell.dll
```

### الترخيص

MIT. راجع [LICENSE](../../LICENSE).
