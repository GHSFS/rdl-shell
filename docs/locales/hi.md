# rdl-shell

<div align="center">

<p align="center">
  <a href="../../README.md">🇺🇸 English</a> · <a href="./ko.md">🇰🇷 한국어</a> · <a href="./zh-CN.md">🇨🇳 中文</a> · <a href="./es.md">🇪🇸 Español</a> · <strong>🇮🇳 हिन्दी</strong><br />
  <a href="./ar.md">🇸🇦 العربية</a> · <a href="./pt-BR.md">🇧🇷 Português</a> · <a href="./ru.md">🇷🇺 Русский</a> · <a href="./fr.md">🇫🇷 Français</a> · <a href="./id.md">🇮🇩 Bahasa Indonesia</a>
</p>

> [remote-dl](https://github.com/GHSFS/remote-dl) के लिए Windows Explorer शेल एक्सटेंशन — `.url` (इंटरनेट शॉर्टकट) फ़ाइलों के राइट-क्लिक मेनू में "Send to remote-dl" विकल्प जोड़ता है।

[![Build](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/build.yml)
[![Tests](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml/badge.svg)](https://github.com/GHSFS/rdl-shell/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](../../LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20x64-blue)](#स्थापना)

</div>

---

### विवरण

`rdl-shell.dll` एक Windows Explorer शेल संदर्भ मेनू हैंडलर है जो [`rdl`](https://github.com/GHSFS/remote-dl) CLI क्लाइंट को सीधे Explorer में एकीकृत करता है। पंजीकृत होने के बाद, `.url` इंटरनेट शॉर्टकट फ़ाइल पर राइट-क्लिक करने पर एक नया मेनू विकल्प — **Send to remote-dl** (या कई फ़ाइलों के चयन पर **Send N shortcuts to remote-dl**) — दिखाई देता है, जो शॉर्टकट से URL पढ़कर कॉन्फ़िगर किए गए वर्कर में कतारबद्ध करता है।

#### ✨ मुख्य विशेषताएं (v0.2.0)
- **मल्टी-फ़ाइल चयन**: एक साथ कई `.url` शॉर्टकट चुनें और एक क्लिक में सभी को कतारबद्ध करें।
- **शून्य Explorer हैंग (Non-Blocking Async)**: पृष्ठभूमि वर्कर थ्रेड और RAII `DllLockGuard` के माध्यम से Explorer UI तुरंत बिना अटके प्रतिक्रिया देता है।
- **सार्वभौमिक शॉर्टकट एन्कोडिंग**: UTF-8, UTF-16LE (BOM सहित/रहित), UTF-16BE और केस-असंवेदनशील `url=` कुंजी का स्वचालित पार्सिंग।
- **सख्त URL सत्यापन**: प्रेषण से पहले केवल `http://` और `https://` योजनाओं की पुष्टि करता है।
- **कनेक्शन पूलिंग अनुकूलन**: Keep-Alive और DNS कैशिंग के साथ कैश किए गए `reqwest` क्लाइंट (`OnceLock`) का पुन: उपयोग।

### स्थापना

> रजिस्ट्री पंजीकरण के लिए व्यवस्थापक (Administrator) विशेषाधिकार आवश्यक हैं।

1. [Releases](https://github.com/GHSFS/rdl-shell/releases) पृष्ठ से `rdl-shell.dll` डाउनलोड करें।
2. इसे एक स्थायी फ़ोल्डर (उदा. `C:\Tools\rdl-shell.dll`) में रखें।
3. व्यवस्थापक के रूप में PowerShell खोलें और चलाएं:

   ```powershell
   regsvr32 C:\Tools\rdl-shell.dll
   ```

4. परिवर्तनों को लागू करने के लिए Explorer को पुनः आरंभ करें।

### हटाना

```powershell
regsvr32 /u C:\Tools\rdl-shell.dll
```

### लाइसेंस

MIT. देखें [LICENSE](../../LICENSE).
