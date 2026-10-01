# SnipHub (22 — Kırpkaynak)

> ## ⚠️ Güvenlik uyarısı — okumadan önce
>
> **Parça deposu düz metindir. Şifreleme yoktur.** `parcalar.json` dosyasındaki
> her şey (`govde`, `aciklama`, `etiketler`) diskte olduğu gibi, okunabilir biçimde
> durur. Bu, `MANIFEST.md` kartındaki **bilinçli sapmanın** sonucudur: Argon2id +
> XChaCha20 kripto crate'leri `WORKER_CONTRACT.md` § 3.2-C uyarınca yalnızca
> 16/17/30 projelerine serbest olduğu için bu projede kullanılamaz.
>
> **Bu aracı parola, API anahtarı, erişim jetonu veya benzeri gizli değer saklamak
> için kullanmayın.** Korunması gereken veri için işletim sistemi disk şifrelemesini
> (BitLocker / LUKS / FileVault) kullanın.

**SnipHub**, `tetik → kod` genişletmesi yapan bir **programlama** parçası
yöneticisidir. Parçalar bir dil etiketiyle saklanır, kaydedildiği anda yapısal
sözdizimi denetiminden geçirilir ve dil + etiket + metin filtreleriyle
bulunur. Kazanç "hissedilen hız" olarak kalmaz, **fiziksel tuş sayacı** ile
ölçülür.

---

## Bu araç neden SnipHub'dır, TypeFast değil

Bu ayrım bilinçli bir ürün yönetimi kararıdır (`MANIFEST.md`, **KARAR D-011**).
Test edilebilir kural şudur:

> **Bir özellik yalnızca "hangi dilde yazıldığı" bilgisine bağlıysa SnipHub'a aittir.**

| Özellik | SnipHub (22) | TypeFast (25) |
|---|---|---|
| Dil etiketi (`rust`, `python`, `sql`, …) | ✅ | ❌ |
| Yapısal sözdizimi denetleyicisi (`check`) | ✅ | ❌ |
| Dil bazlı arama ve filtre | ✅ | ❌ |
| `tetik → kod` genişletme | ✅ | ✅ |
| Genel metin şablonları (programlama bilgisi gerektirmez) | ❌ | ✅ |
| Pano geçmişi | ❌ | ✅ |
| Gizli mod (`!gizli`) | ❌ | ✅ |
| Genel kısayol sözlüğü | ❌ | ✅ |
| Şifreleme | ❌ (düz metin) | ❌ (düz metin) |

**Bu depoda bilinçli olarak uygulanmayanlar:** pano geçmişi, gizli mod, genel
metin şablonları, şifreli depo, global kısayol FFI'si. Bunlar 25 TypeFast'ın
alanıdır ve burada tekrarlanmaz. `Dil::Metin` çeşidi yalnızca bir etikettir ve
yapısal denetime girmez.

---

## Özellikler

- **Parça deposu** — benzersiz ad, tetik, açıklama, dil etiketi, etiket listesi,
  kod gövdesi, oluşturma/güncelleme damgası, kullanım sayacı ve denetim durumu.
  Biçim sürümlüdür (`surum: 1`) ve şema denetiminden geçer.
- **Genişleme dili** — `{{imleç}}`, `{{secim}}`, konumsal `{{1}}`, adlı `{{ad}}`,
  varsayılanlı `{{1|limit|100}}` ve iç içe çözülebilir varsayılanlar.
  Yerleşikler: `{{tarih}}`, `{{saat}}`, `{{zaman}}`, `{{kullanici}}`.
  Kaçış: `\{{` ve `\}}`.
- **Yapısal sözdizimi denetleyicisi** — Rust ve Python için el yazımı tarayıcı:
  ayraç dengesi, tırnak/escape kapanışı, ham dizeler, karakter sabiti, yaşam
  süresi belirteci, makro gövdesi ve (Python için) girinti tutarlılığı.
  Bulgu satır/sütnumarası ve açıklamayla döner.
- **Çoklu dil arama** — metin + dil + etiket filtreleri, **kararlı** alaka
  sıralaması, puan gerekçeleri. Denetimi geçemeyen parça son sırada çıkar.
- **Ölçülebilir kazanç** — gerçek fiziksel tuş sayacı (büyük harf = `Shift`+tuş),
  ortak girinti arındırma, WPM tabanlı saniye karşılığı.
- **Dışa/içe aktarma** — JSON, Markdown (gidiş-dönüşlü) ve **tek dosya** HTML
  katalog (hiçbir dış kaynak referansı yok).
- **Atomik yazma** — geçici dosya + `fs::rename`; yazma sırasında çökme eski
  depoyu bozmaz.
- **Çakışma politikası** — `hata`, `atla`, `degistir`, `yeniden-adlandir`.

---

## Kurulum

**MSRV:** Rust 1.74 · Bu README'deki tüm çıktılar **rustc 1.98.1** ile üretilmiştir.

```console
$ cargo build --release
   Compiling sniphub v0.1.0 (%USERPROFILE%\Desktop\Projeler\projects\22-sniphub)
    Finished `release` profile [optimized] target(s) in 6.08s
```

İkili doğrudan çalıştırılabilir:

```console
$ .\target\release\sniphub.exe --version
sniphub 0.1.0
```

Cargo ile kurmak isteyenler için:

```console
$ cargo install --path .
```

> **PowerShell notu:** Tetikler `;` ile başladığı için Windows PowerShell 5.1'de
> tırnak içine alınmalıdır: `sniphub expand ";sql-sorgu" --set "1=id"`.
> Tırnaksız yazılırsa PowerShell `;` karakterini komut ayırıcı sanar.

---

## Kullanım

Aşağıdaki tüm çıktılar `ornek/` altındaki 18 parçalık gerçek koleksiyondan
alınmıştır. Koleksiyonu yeniden üretmek için:

```console
$ powershell -NoProfile -ExecutionPolicy Bypass -File ornek\yukle-ornek.ps1
example repo written: %USERPROFILE%\Desktop\Projeler\projects\22-sniphub\ornek\parcalar.json
parça sayısı     : 18
...
```

> Not: `ornek/yukle-ornek.ps1` betiği **UTF-8 BOM** ile kaydedilmiştir. BOM'suz
> bir `.ps1` dosyasını Windows PowerShell 5.1 ANSI kodlamasıyla okur ve Türkçe
> karakterleri bozar.

### `list` — depo içindeki parçalar

```console
$ sniphub --depo ornek\parcalar.json list
Python CSV okuyucu         python     ;py-csv         0 kullanım  gecerli (python, csv, veri)
Python dataclass           python     ;py-dataclass    0 kullanım  gecerli (python, veri, iskelet)
Python dosya okuma         python     ;py-oku         0 kullanım  gecerli (python, dosya, okuma)
Python hata yakalama       python     ;py-hata        0 kullanım  gecerli (python, hata)
Python liste donusumu      python     ;py-liste       0 kullanım  gecerli (python, anlayis)
Python sozluk siralama     python     ;py-sirala      0 kullanım  gecerli (python, siralama, lamba)
Rust Display uygulamasi    rust       ;rust-display    0 kullanım  gecerli (rust, trait, fmt)
Rust dosya okuma           rust       ;rust-dosya     0 kullanım  gecerli (rust, dosya, io)
Rust hata yakalama         rust       ;rust-hata      0 kullanım  gecerli (rust, hata, iskelet)
Rust iterator zinciri      rust       ;rust-iter      0 kullanım  gecerli (rust, iterasyon)
Rust match guard           rust       ;rust-match     0 kullanım  gecerli (rust, match, iskelet)
Rust struct tanimi         rust       ;rust-struct    0 kullanım  gecerli (rust, veri, iskelet)
Rust test fonksiyonu       rust       ;rust-test      0 kullanım  gecerli (rust, test, iskelet)
SQL guncelleme blogu       sql        ;sql-guncel     0 kullanım  denenmemis (sql, guncelleme)
SQL sorgu iskeleti         sql        ;sql-sorgu      0 kullanım  denenmemis (sql, sorgu, iskelet)
SQL tarih filtresi         sql        ;sql-tarih      0 kullanım  denenmemis (sql, tarih, sorgu)
Python bozuk girinti       python     ;py-bozuk       0 kullanım  hatali (python, test, bozuk)
Rust bozuk ornek           rust       ;rust-bozuk     0 kullanım  hatali (rust, test, bozuk)
```

Üç denetim durumu birbirinden ayrıdır: `gecerli` (denetlendi ve temiz),
`hatali` (denetlendi ve sorun bulundu), `denenmemis` (denetleyici bu dil için
kurallı değil — SQL için geçerlidir).

### `expand` — yer tutucuları doldurma

```console
$ sniphub --depo ornek\parcalar.json expand ";sql-sorgu" --set "1=id, ad" --set "2=kayitlar" --set "3=durum = 'aktif'" --set "4=id" --set "5=25"
yuva #1 `kolonlar` satır 1 sütun 8 [doldu] = id, ad
yuva #2 `tablo` satır 2 sütun 6 [doldu] = kayitlar
yuva #3 `kosul` satır 3 sütun 7 [doldu] = durum = 'aktif'
yuva #4 `siralama` satır 4 sütun 10 [doldu] = id
yuva #5 `limit` satır 5 sütun 7 [doldu] = 25
-----
SELECT id, ad
FROM kayitlar
WHERE durum = 'aktif'
ORDER BY id DESC
LIMIT 25;
```

Doldurulmayan yer tutucular varsayılan değerini alır:

```console
$ sniphub --depo ornek\parcalar.json expand ";sql-tarih" --set "1=olaylar" --sayac falseyuva #1 `tablo` satır 1 sütun 15 [doldu] = olaylar
yuva #2 `tarih` satır 2 sütun 21 [varsayılan] = bugun
yuva #3 `durum` satır 3 sütun 16 [varsayılan] = aktif
-----
SELECT * FROM olaylar
WHERE olusturma >= 'bugun'
  AND durum = 'aktif'
```

Yerleşikler sistem saatinden ve ortam değişkeninden doğar:

```console
$ sniphub --depo ornek\parcalar.json expand ";py-oku" --set "1=rapor_oku" --set "2=aylik"
yuva #1 `oku` satır 1 sütun 5 [doldu] = rapor_oku
yuva #2 `dosya` satır 2 sütun 8 [doldu] = aylik
-----
def rapor_oku(yol: str) -> str:
    """aylik dosyasını metin olarak okur."""
    with open(yol, encoding="utf-8") as dosya:
        return dosya.read()
```

### `check` — yapısal sözdizimi denetimi

Depodaki bir parçayı denetlemek:

```console
$ sniphub --depo ornek\parcalar.json check ";rust-bozuk"
Rust · denge tamam değil · 2 bulgu
  1:12: hata [acik-ayrac] süslü parantez `{` açıldı ama kapatılmadı
  3:1: hata [eslesmeyen-kapanis] `}` kapanışı, satır 2 sütun 17'de açılan `[` ile eşleşmiyor
```

```console
$ sniphub --depo ornek\parcalar.json check ";py-bozuk"
Python · denge tamam değil · 1 bulgu
  4:1: hata [girinti] girinti 6, yığında eşleşen bir düzey bulamadı (tepe 8)
```

Temiz kod sessiz geçer:

```console
$ sniphub --depo ornek\parcalar.json check ";py-oku"
Python · denge tamam · 0 bulgu
  yapısal denge: temiz
```

`stdin`'den okuyarak da çalışır (pano FFI'si olmadan betiklenebilir kullanım):

```console
$ "def f(:" | sniphub check --lang python
Python · denge tamam değil · 1 bulgu
  1:6: hata [acik-ayrac] `(` açıldı ama kapatılmadı
```

### `search` — dil + etiket + metin

```console
$ sniphub --depo ornek\parcalar.json search veri --lang rust
2 sonuç:
  [rust] Rust struct tanimi       ;rust-struct puan   50  etiket tam eşleşti
  [rust] Rust dosya okuma         ;rust-dosya puan   17  gövdede geçiyor
```

```console
$ sniphub --depo ornek\parcalar.json search iskelet --limit 4
6 sonuç:
  [sql] SQL sorgu iskeleti       ;sql-sorgu puan  120  ad içinde geçiyor, etiket tam eşleşti, açıklamada geçiyor, kimlikte geçiyor
  [rust] Rust hata yakalama       ;rust-hata puan   65  etiket tam eşleşti, açıklamada geçiyor
  [rust] Rust match guard         ;rust-match puan   65  etiket tam eşleşti, açıklamada geçiyor
  [rust] Rust struct tanimi       ;rust-struct puan   65  etiket tam eşleşti, açıklamada geçiyor
```

Her satır puanın **neden** oluştuğunu gösterir. Sıralama kararlıdır: puan
eşitse ada, ada eşitse kimliğe göre sıralanır.

### `get` — tek parça ve ölçülen kazancı

```console
$ sniphub --depo ornek\parcalar.json get ";rust-hata"
ad        : Rust hata yakalama
id        : rust-hata-yakalama-c6c27e00
tetik     : ;rust-hata
dil       : Rust
etiketler : rust, hata, iskelet
aciklama  : Result tabanli hata donduren fonksiyon iskeleti.
olusturma : 2026-09-29T17:12:15Z
guncelleme: 2026-09-29T17:12:15Z
kullanim  : 0
kazanc    : 171 tuş (51.30 saniye)

-----
pub fn {{1|islem}}(girdi: &str) -> Result<String, Hata> {
    if girdi.is_empty() {
        return Err(Hata::BosGirdi);
    }
    Ok(girdi.to_uppercase())
}
-----
```

### `export` / `import`

```console
$ sniphub --depo ornek\parcalar.json export --format markdown | Select-Object -First 16
# SnipHub Katalogu

<!-- surum: 1 | parca: 18 -->

## `Python bozuk girinti`

- tetik: `;py-bozuk`
- dil: `python`
- etiket: `python, test, bozuk`
- kullanim: `0`
- dogrulama: `hatali`

> Girinti denetimini tetiklemek icin bilinen hatali girinti.

```python
def {{1|kontrol}}(x):
    if x > 0:
        return "pozitif"
      return "negatif"
```
```

```console
$ sniphub --depo ornek\parcalar.json export --format html --out ornek\katalog.html
18 parça html biçiminde yazıldı: ornek\katalog.html (11077 bayt)
$ sniphub --depo ornek\parcalar.json export --format markdown --out ornek\katalog.md
18 parça markdown biçiminde yazıldı: ornek\katalog.md (5990 bayt)
$ sniphub --depo ornek\parcalar.json export --format json --lang python --out ornek\python-koleksiyonu.json
7 parça json biçiminde yazıldı: ornek\python-koleksiyonu.json (4165 bayt)
```

Markdown biçimi **gidiş-dönüşlüdür**: dışa aktarılan katalog doğrudan geri
içe aktarılabilir. Aynı adla içe aktarımda çakışma politikası uygulanır:

```console
$ sniphub --depo ornek\parcalar.json import ornek\katalog.md --format markdown --on-conflict hata
hata: bu adda bir parça zaten var: Python bozuk girinti

$ sniphub --depo $env:TEMP\bos-depo.json import ornek\katalog.md --format markdown
içe aktarıldı: 18 eklendi, 0 değişti, 0 yeniden adlandırıldı, 0 atlandı (politika: hata)
```

### `stats` — ölçülebilir kazanç

```console
$ sniphub --depo ornek\parcalar.json stats --top 3
parça sayısı     : 18
toplam genişletme : 2
kazanan tuş      : 324
kazanan süre      : 97.2 saniye (1.6 dakika)
hız varsayımı     : 40 WPM
kazancı olmayan    : 0 parça (%0)

dil dağılımı:
  Python       7
  Rust         8
  SQL          3

en çok kullanılan 2:
  Python dosya okuma               1 kez      159 tuş/kayıt        159 tuş toplam
  SQL sorgu iskeleti               1 kez      165 tuş/kayıt        165 tuş toplam
```

**Bu sayılar nasıl üretiliyor?** Gövde, ortak girinti payı arındırıldıktan sonra
her karakter için fiziksel tuş vuruşu sayılır (küçük harf/rakam/boşluk = 1,
büyük harf ve `Shift` simgeleri = 2, satır sonu = 1, sekme = 1). Kazanç =
gövde tuşları − (tetik tuşları + 1 ayraç boşluğu). Süre karşılığı
`kazanc_tus / (wpm × 5 / 60)` formülüyle hesaplanır.

---

## Test

```console
$ cargo test
running 115 tests
...
test result: ok. 115 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests\entegrasyon.rs
running 10 tests
...
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
```

**test sonucu: okunan 128; başarısız 0** (115 birim + 10 entegrasyon + 3 CLI ayrıştırma).

Kapsanan konu başlıkları:

| Alan | Örnekler |
|---|---|
| Genişleme söz dizimi | imleç, seçim, konumsal/adlı/varsayılanlı belirteç, **iç içe** çözümleme, `\{{` kaçışı, kapanmamış `{{`, aşırı `}}`, geçersiz belirteçler, adım tavanı |
| Sözdizimi denetleyicisi — Rust | parantez/süslü/köşeli denge, eşleşmeyen kapanış, tırnak escape, kapanmamış dizge, **yaşam süresi** belirteci, karakter sabiti, **makro** gövdesi, ham dizge, blok yorum |
| Sözdizimi denetleyicisi — Python | üçlü tırnak (açık/kapalı), **ham dizgede** kaçış işlenmemesi, girinti hatası, girintisiz gövde, parantez içi devam, yorum içi ayraç |
| Arama / filtre / sıralama | dil filtresi, etiket AND/OR, puan gerekçeleri, kararlı sıralama, büyük/küçük harf |
| Kazanç hesabı | tuş sayacı kuralları, girinti arındırma, WPM ölçekleme, kazancı olmayan parça |
| JSON şema gidiş-dönüşü | `Depo` → JSON → `Depo` birebir |
| MD / HTML dışa aktarma | Markdown gidiş-dönüşü, gövde içi çit, HTML kaçışı, dış kaynak yokluğu |
| İçe aktarma çakışma | `hata` / `atla` / `degistir` / `yeniden-adlandir` (son ek artışı) |
| Bozuk depo dosyası | bozuk JSON, boş dosya, desteklenmeyen sürüm, şema ihlali |
| Yazma atomikliği | geçici dosya kalmaması, geçersiz şemanın diske yazılmaması |
| Aynı ad iki parça | ikinci ekleme reddedilir, diskteki depo tek parça kalır |
| Kazara şifreli içerik | depo düz metindir, şifreleme alanı yok |
| Zaman dönüşümü | epoch, artık yıl, 2000-03-01, 2026-09-29 sabit vektörleri |
| CLI ayrıştırma | bayrak çakışması (`--body` + `--body-file`), değer türleri |
| İkili entegrasyonu | `list`/`search`/`expand`/`check`/`get` gerçekten çalışır, hata senaryosu sıfır dönüş kodu verir |

Testler ağ kullanmaz, rastgelelik kullanmaz ve geçici dosya yardımcısını
`Drop` ile temizler (`tempfile` bağımlılığı yoktur).

---

## Proje Yapısı

```
22-sniphub/
├── Cargo.toml
├── Cargo.lock
├── LICENSE.txt
├── README.md
├── .gitignore
├── src/
│   ├── lib.rs           çekirdek kütüphane, #![forbid(unsafe_code)]
│   ├── main.rs          clap CLI kabuğu
│   ├── hata.rs          tek hata tipi, elle Display + Error
│   ├── zaman.rs         std::time tabanlı ISO-8601 dönüşümü
│   ├── dil.rs           dil etiketleri, uzantı → dil çözümlemesi
│   ├── model.rs         Parca / Depo şeması, saf iş kuralları
│   ├── genisle.rs       {{belirtec}} genişleme dili
│   ├── syntax.rs        Rust + Python yapısal denetleyicisi
│   ├── depo.rs          atomik okuma/yazma, biçim sürümü
│   ├── arama.rs         arama, filtre, kararlı alaka sıralaması
│   ├── kazanc.rs        fiziksel tuş sayacı tabanlı kazanç
│   ├── aktarma.rs       JSON / Markdown / HTML dışa-içe aktarma
│   └── istatistik.rs    kullanım ve kazanç raporu
├── tests/
│   └── entegrasyon.rs   uçtan uca akış + gerçek ikili çalıştırma
├── ornek/
│   ├── yukle-ornek.ps1  örnek koleksiyonu üreten betik
│   ├── parcalar.json    aracın kendi ürettiği 18 parçalık depo
│   ├── katalog.md       Markdown dışa aktarımı
│   ├── katalog.html     tek dosya HTML katalog
│   ├── python-koleksiyonu.json
│   └── govdeler/        parça gövdeleri (UTF-8)
└── target/              (derleme çıktısı, commit edilmez)
```

---

## Yapılandırma

Yapılandırma dosyası veya ortam değişkeni **yoktur**. Tüm ayarlar bayrakla verilir.

| Bayrak / Değişken | Varsayılan | Etkisi |
|---|---|---|
| `--depo <YOL>` | çalışma dizinindeki `parcalar.json` | Depo dosyası yolu. Tüm komutlarda geçerlidir. |
| `add --name <AD>` | zorunlu | Parçanın benzersiz adı. Aynı ad iki parça olamaz. |
| `add --trigger <TETİK>` | zorunlu | `expand` tetiği. |
| `add --lang <DİL>` | zorunlu | `rust`, `python`, `javascript`, `sql`, `json`, `toml`, `markdown`, `metin`. |
| `add --desc <AÇIKLAMA>` | boş | Tek satırlık açıklama. |
| `add --tag <a,b,c>` | boş | Virgülle ayrılmış etiketler. |
| `add --body <METİN>` / `add --body-file <YOL>` | — | Gövde. İkisinden **yalnızca biri** verilir. |
| `search --lang <DİL>` | yok | Yalnızca bu dildeki parçalar. |
| `search --tag <a,b>` + `--her-etiket` | `--her-etiket` kapalı | Etiket filtresi; bayrak açıkken AND, kapalıyken OR. |
| `search --limit <N>` | `20` | En fazla gösterilecek sonuç. |
| `expand --set <ANAHTAR=DEĞER>` | yok | Yer tutucu ataması, tekrarlanabilir. `1` konumsal, `ad` adlı. |
| `expand --sayac <true\|false>` | `true` | Kullanım sayacını artırıp artırmama. |
| `check <ANAHTAR>` | — | Depodaki parçayı denetler. Verilmezse `stdin` okunur. |
| `check --lang <DİL>` | parçanın dili | `stdin` kipinde zorunludur. |
| `export --format <json\|markdown\|html>` | `json` | Çıktı biçimi. |
| `export --lang <DİL>` | yok | Yalnızca bu dili aktar. |
| `export --out <YOL>` | stdout | Yazılacak dosya. |
| `import --on-conflict <poltika>` | `hata` | `hata`, `atla`, `degistir`, `yeniden-adlandir`. |
| `stats --top <N>` | `10` | En çok kullanılan kaç parça. |
| `stats --wpm <N>` | `40` | Yazma hızı (WPM); kazanç süresi buna göre ölçeklenir. |
| `VARSAYILAN_DEPO_ADI` | `parcalar.json` | Depo dosyasının varsayılan adı (sabit). |
| `ADIM_TAVANI` | `32` | Bir genişlemede çözülebilecek en fazla belirteç sayısı. |
| `ICICE_TAVANI` | `8` | Varsayılan değer içinde izin verilen iç içe çözümleme derinliği. |
| `VARSAYILAN_WPM` | `40.0` | Tuş → saniye dönüşümünde kullanılan hız. |
| `DEPO_SURUMU` | `1` | Desteklenen depo biçim sürümü. |

### Depo dosyası şeması

```json
{
  "surum": 1,
  "parcalar": [
    {
      "id": "sql-sorgu-iskeleti-1a2b3c4d",
      "ad": "SQL sorgu iskeleti",
      "tetik": ";sql-sorgu",
      "aciklama": "Konumsal ve varsayılan degerli yer tutucular tasiyan SQL iskeleti.",
      "dil": "sql",
      "etiketler": ["sql", "sorgu", "iskelet"],
      "govde": "SELECT {{1|kolonlar}}\nFROM {{2|tablo}}\n...",
      "olusturma": "2026-09-29T17:11:52Z",
      "guncelleme": "2026-09-29T17:13:40Z",
      "kullanim_sayaci": 1,
      "dogrulama": "denenmemis"
    }
  ]
}
```

`id`, ada + gövdeye uygulanan FNV-1a sağlamasından türetilir; rastgelelik
içermez, bu yüzden `add` komutu idempotenttir.

---

## Bilinen Sınırlamalar

**Bu bölüm bilinçlidir ve eksiksizdir. `MANIFEST.md` kartındaki "Ertelenen"
listesi ile eşleşir.**

### Güvenlik

1. **Şifreleme yoktur.** Depo düz metin JSON'dır. Argon2id + XChaCha20 kapsam
   dışıdır (kripto crate'leri yalnız 16/17/30 projelerine serbest). Bu, ürünün
   en önemli güvenlik eksiğidir.
2. **Gizli mod ve pano geçmişi yoktur.** Bunlar 25 TypeFast'ın alanıdır (D-011).
3. **Pano dinleme ve global kısayol FFI'si yoktur.** Bu yollar Rust `std`'de
   yoktur ve `unsafe` gerektirir; WORKER_CONTRACT § 3.2-G pencere/grafik
   katmanını yasaklar.

### Sözdizimi denetleyicisi

4. **Gerçek dil grameri değildir.** Yalnızca ayraç dengesi, tırnak/escape
   kapanışı ve (Python'da) girinti tutarlılığı denetlenir. "Doğrulandı" değil,
   **"yapısal denge kontrol edildi"** denir. `syn`, `ruff` ve tree-sitter
   gramerleri yasaktır.
5. **Yalnızca Rust ve Python denetlenir.** `javascript`, `sql`, `json`, `toml`
   ve `markdown` için uzantı çözümlemesi vardır ama kural yoktur; bu parçalar
   `denenmemis` olarak işaretlenir. Gerçek tree-sitter gramerleri ertelenmiştir.
6. **Rust makro gövdeleri bağlama duyarlıdır.** `println!("{")` gibi kalıplarda
   denetleyici ayraç sayımını yapısal kural uygular ve `kapanmamis-makro`
   bulgusu üretir. Bu bir bilinçli yaklaşımdır, kusursuz bir çözüm değildir.
7. **Yaşam süresi / karakter sabiti ayrımı bağlama duyarlıdır.** Geçerli
   Rust'ta yaşam süresinden sonra `;` gelemeyeceği bu ayrımda kullanılır; bu
   gürültü üretmez ama kural tabanlıdır, derleyici değildir.
8. **Python'da karışık sekme/boşluk denetlenmez.** `TAB` sekizlik sütun adımıyla
   sayılır.

### Arama

9. **Büyük/küçük harf dönüşümü Unicode varsayılan eşlemesini kullanır.** ASCII
   `I` harfini `i` yapar; Türkçe'de `ı` beklenir. `Başlığı` aranırsa
   `BAŞLIĞI` ile eşleşmez. Bu, teste bağlı olarak belgelenmiş bir sınırdır
   (`turkce_noktasiz_i_buyuk_kucuk_harf_sinirlamasi_belgelidir`).
10. **Tam metin indeksi yoktur.** Arama doğrusal taramadır ve 20.000 parçada
    ölçülmemiştir. Gövde içinde geçen metin en düşük puanla eşleşir.

### Kazanç ölçümü

11. **Klavye düzeni modellenmez.** US-QWERTY varsayılır; Türkçe/Farklı düzen
    kullanıcıları için sayılar kayar.
12. **Ölü tuşlar, IME ve otomatik girintileme modellenmez.** Türkçe harfler
    (ç, ğ, ı, ö, ş, ü) 1 tuş sayılır.
13. **WPM 40 bir varsayımdır**, kişisel ölçümle değiştirilebilir
    (`stats --wpm`). Kazanç saniyesi bu varsayıma bağlıdır.
14. **Ölçüm kendisi henüz kullanıcı çalışmasıyla doğrulanmamıştır.** Fikir
    raporu § 08'in uyarısı geçerlidir: "hissedilen hız" ölçülmüş bir sayı
    değildir. Bu aracın ürettiği sayı **temsilî** bir hesaptır.

### Aktarma

15. **HTML katalog tek dosyadır ama JS içermez.** Arama/filtre etkileşimi
    sunulmaz; statik bir katalogdur.
16. **Markdown dışa aktarımı içe aktarılabilir**, HTML içe aktarılamaz
    (bilinçli karar: HTML çıktısı bir *katalog*, bir *paket* değildir).
17. **İçe aktarımda biçim sürümü denetlenmez**; sürüm uyuşmazlığı şema
    denetiminde yakalanır.

### Diğer

18. **Tetik çakışması çözümü yoktur.** İki parça aynı tetiği taşırsa
    `expand` ad/tetik sırasına göre ilk eşleşeni kullanır. Bağlama göre
    önceliklendirme (v1) ertelenmiştir.
19. **Dosya tabanlı takım paketi ve bütünlük özeti yoktur.** Paylaşım,
    düz `parcalar.json` kopyası veya Markdown/JSON dışa aktarımı üzerinden
    elle yapılır.
20. **`target/` dışında hiçbir yere yazılmaz.** Depo yolu açıkça verilir.
21. **Yapılandırma dosyası yoktur.** Sezgiye aykırı olabilir ama tek dosya
    dağıtımı ve yol bağımsızlığı hedefiyle tutarlıdır.
22. **Kapsam kayması riski (D-011) bilinçlidir.** 25 TypeFast ile yakınlığı
    nedeniyle "genel metin" özellikleri burada **uygulanmamıştır**; bu, eksik
    değil kasıtlı bir ayrımdır.

### `#[allow]` kullanımları

Yalnızca bir yerde `#[allow]` vardır ve gerekçesi buradadır:
`src/aktarma.rs` → `parca_bitir` fonksiyonunda `clippy::too_many_arguments`
(8 parametre; Markdown ayrıştırıcısının durum taşıyıcısı).

`#![forbid(unsafe_code)]` gevşetilmemiştir. `src/lib.rs` ve `src/main.rs` en
üstünde `#![forbid(unsafe_code)]` ve `#![deny(missing_docs)]` vardır.
`clippy::unwrap_used` / `clippy::expect_used` uyarıları
`#![cfg_attr(not(test), warn(...))]` ile yalnızca üretim kodunda etkindir;
testler bunları meşru olarak kullanır.

---

## Gelecek Geliştirmeler

`MANIFEST.md` kartındaki ertelenenler, doğal sıralamayla:

1. **Tetik çakışması çözümü** — öncelik listesi, bağlama göre kısıt ve
   belirsizlik raporu.
2. **Dosya tabanlı takım paketi** — bütünlük özeti ve sürüm bilgisiyle
   imzalı paket; özet uyuşmazsa içe aktarma durur.
3. **Ek dil desteği** — JavaScript/TypeScript için aynı yapısal denetleyici
   kalıbı.
4. **Parça versiyon geçmişi** — gövde değiştiğinde eski sürüm saklanır.
5. **Genişleme oturumu** — sıralı odaklanma, `sonraki`/`önceki` komutları ve
   boş bırakılan alanın varsayılana dönmesi (rapor § 03, senaryo S2).
6. **Mimari gramer desteği** — gerçek tree-sitter gramerleri (kapsam dışı
   karar gerektirir; bağımlılık politikasıyla uyumsuz).
7. **Türkçe büyük/küçük harf duyarlı arama** — noktasız/dotlu `i` ayrımı.
8. **Hızlı arama indeksi** — 20.000 parça ölçeğinde doğrulanmış.

---

## Troubleshooting

### 1. `linker 'link.exe' not found` (Windows)

**Belirti:** `cargo build` derleme sırasında bağlama hatası verir.
**Neden:** MinGW / MSVC bağlayıcısı `PATH` üzerinde değildir.
**Çözüm:** Her `cargo` çağrısından önce `PATH`'e MinGW dizinini ekleyin:

```powershell
$env:PATH = "%USERPROFILE%\.cargo\bin;%USERPROFILE%\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin;" + $env:PATH
```

### 2. Tetik `;` ile başlıyor ve PowerShell hata veriyor

**Belirti:** `expand ;sql-sorgu` → *"The term 'sql-sorgu' is not recognized"*.
**Neden:** PowerShell 5.1 `;` karakterini komut ayırıcı olarak yorumlar.
**Çözüm:** Tetiği tırnak içine alın: `sniphub expand ";sql-sorgu" --set "1=id"`.

### 3. `.ps1` betiğindeki Türkçe karakterler bozuk görünüyor

**Belirti:** Betik çalışır ama gövde metinlerinde `dosyasÄ±nÄ±` gibi
harfler belirir.
**Neden:** Windows PowerShell 5.1, BOM'suz `.ps1` dosyasını ANSI kodlamasıyla
okur.
**Çözüm:** Betiği **UTF-8 BOM** ile kaydedin. `ornek/yukle-ornek.ps1` bu
şekilde kaydedilmiştir; gövdeler ayrıca `--body-file` ile UTF-8 dosyalardan
okunur, böylece iki katmanlı bir koruma sağlanır.

### 4. `hata: bu adda bir parça zaten var`

**Belirti:** `add` veya `import --on-conflict hata` başarısız olur.
**Neden:** Depoda aynı adda bir parça var; adlar büyük/küçük harf duyarsız
karşılaştırılır.
**Çözüm:** `list` ile mevcut adı görün, ya da `import --on-conflict degistir`
veya `--on-conflict yeniden-adlandir` kullanın.

### 5. `check` hiçbir şey bulmuyor ama kod hatalı

**Belirti:** Temiz görünen kod `denge tamam` döndürüyor.
**Neden:** Denetleyici gerçek gramer değildir; tip hatası, ad çözümlemesi ve
anlamsal kural denetlenmez.
**Çözüm:** `check` yalnızca **yapısal denge** kontrolüdür. Gerçek doğrulama için
`cargo build` / `ruff` gibi harici araçlar kullanın.

### 6. `desteklenmeyen depo sürümü 2 (bu sürüm 1 sürümünü okur)`

**Belirti:** Depo okunamıyor.
**Neden:** Dosya bu programdan daha yeni bir sürümle yazılmış.
**Çözüm:** Dosyayı yedekleyin, `surum` alanını geri alın veya programı
güncelleyin. SnipHub biçim sürümü ilerlemelerini reddeder; sessizce bozuk
veri okumaz.

---

## Atıflar

- **Rust standart kütüphane belgeleri** — <https://doc.rust-lang.org/std/>
- **Rust 2021 edition rehberi** — <https://doc.rust-lang.org/edition-guide/edition-2021/>
- **`cargo` yerel rehberi** — <https://doc.rust-lang.org/cargo/>
- **`serde`** — <https://serde.rs/> · <https://docs.rs/serde/>
- **`serde_json`** — <https://github.com/serde-rs/json> · <https://docs.rs/serde_json/>
- **`clap`** — <https://docs.rs/clap/> · <https://github.com/clap-rs/clap>
- **Howard Hinnant, "chrono-Compatible Low-Level Date Algorithms"** —
  <https://howardhinnant.github.io/date_algorithms.html>
  (`src/zaman.rs` içindeki `civil_from_days` / `days_from_civil` bu makalenin
  algoritmasını uygular.)
- **FNV (Fowler–Noll–Vo) karma** — <https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function>
  (`src/model.rs` içindeki `fnv1a64`.)
- **Fikir raporunun kendisi:**
  `%USERPROFILE%\Desktop\Fikirler\22-kirp-kaynak-snippet-merkezi.html`
  (yerel yol; iç tasarımın kaynağıdır)
- **Karar D-011 ve proje kartı:**
  `%USERPROFILE%\Desktop\Projeler\MANIFEST.md` (yerel yol)
- **Bağlayıcı sözleşme:**
  `%USERPROFILE%\Desktop\Projeler\WORKER_CONTRACT.md` (yerel yol)

Doğrudan kopyalanmış üçüncü taraf kodu **yoktur**.

---

## Üretim Atfı

Bu depo **OpenCode** ajanı tarafından, **`space-bunny-free`** modeli
(`opencode/space-bunny-free`) kullanılarak üretilmiştir.

- **Arac:** OpenCode
- **Model:** `opencode/space-bunny-free` (Space Bunny Free)
- **Tür:** Rust, `cargo build` / `cargo test` ile üretilmiş ve doğrulanmıştır.

Kaynak kod, testler ve dokümantasyon bu model tarafından yazılmıştır. İnsan
katkısı: gereksinim tanımı, kabul ölçütleri ve son kontroller.

## Lisans

MIT — tam metin için [`LICENSE.txt`](LICENSE.txt) dosyasına bakın.
Kaynak kod lisansı MIT'tir (`MANIFEST.md` D-003).
