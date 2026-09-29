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

## `Python CSV okuyucu`

- tetik: `;py-csv`
- dil: `python`
- etiket: `python, csv, veri`
- kullanim: `0`
- dogrulama: `gecerli`

> csv.DictReader ile sozluk listesine donusturen fonksiyon.

```python
import csv

def {{1|yukle}}(yol: str) -> list[dict]:
    with open(yol, newline="", encoding="utf-8") as dosya:
        return list(csv.DictReader(dosya))
```

## `Python dataclass`

- tetik: `;py-dataclass`
- dil: `python`
- etiket: `python, veri, iskelet`
- kullanim: `0`
- dogrulama: `gecerli`

> Varsayilan degerli dataclass tanimi.

```python
from dataclasses import dataclass

@dataclass
class {{1|Kayit}}:
    {{2|ad}}: str
    {{3|deger}}: int = 0
```

## `Python dosya okuma`

- tetik: `;py-oku`
- dil: `python`
- etiket: `python, dosya, okuma`
- kullanim: `1`
- dogrulama: `gecerli`

> with deyimi ve ucle tirnakli dokustrum iceren fonksiyon.

```python
def {{1|oku}}(yol: str) -> str:
    """{{2|dosya}} dosyasını metin olarak okur."""
    with open(yol, encoding="utf-8") as dosya:
        return dosya.read()
```

## `Python hata yakalama`

- tetik: `;py-hata`
- dil: `python`
- etiket: `python, hata`
- kullanim: `0`
- dogrulama: `gecerli`

> except blogu ve f-string mesaji.

```python
def {{1|calistir}}(is_levi):
    try:
        return is_levi()
    except {{2|TipHata}} as hata:
        print(f"hata: {hata}")
        raise
```

## `Python liste donusumu`

- tetik: `;py-liste`
- dil: `python`
- etiket: `python, anlayis`
- kullanim: `0`
- dogrulama: `gecerli`

> Liste ve sozluk kapsayici anlayislari.

```python
liste = [{{1|eleman}} for oge in {{2|kaynak}} if {{3|kosul}}]
sozluk = {oge[0]: oge[1] for oge in {{2|kaynak}}}
```

## `Python sozluk siralama`

- tetik: `;py-sirala`
- dil: `python`
- etiket: `python, siralama, lamba`
- kullanim: `0`
- dogrulama: `gecerli`

> lambda ile cok satirlik sorted cagrisi.

```python
def {{1|sirala}}(kayitlar: list[dict]) -> list[dict]:
    return sorted(
        kayitlar,
        key=lambda k: k.get("{{2|alan}}", ""),
        reverse={{3|ters|False}},
    )
```

## `Rust bozuk ornek`

- tetik: `;rust-bozuk`
- dil: `rust`
- etiket: `rust, test, bozuk`
- kullanim: `0`
- dogrulama: `hatali`

> Yapisal denge kontrolunun yakalamasi icin bilerek kapanmamis ayrac.

```rust
fn bozuk() {
    let v = vec![1, 2, 3;
}
```

## `Rust Display uygulamasi`

- tetik: `;rust-display`
- dil: `rust`
- etiket: `rust, trait, fmt`
- kullanim: `0`
- dogrulama: `gecerli`

> Display trait uygulamasi; imlec konumu isaretlidir.

```rust
impl std::fmt::Display for {{1|Tip}} {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{2|okuma}}", self.{{3|alan}}){{imlec}}
    }
}
```

## `Rust dosya okuma`

- tetik: `;rust-dosya`
- dil: `rust`
- etiket: `rust, dosya, io`
- kullanim: `0`
- dogrulama: `gecerli`

> Argumandan yol alip dosyayi String olarak okuyan fonksiyon.

```rust
fn {{1|okuma|oku}}() -> std::io::Result<String> {
    let yol = std::env::args().nth(1).unwrap_or_else(|| "veri.txt".to_string());
    Ok(std::fs::read_to_string(&yol)?)
}
```

## `Rust hata yakalama`

- tetik: `;rust-hata`
- dil: `rust`
- etiket: `rust, hata, iskelet`
- kullanim: `0`
- dogrulama: `gecerli`

> Result tabanli hata donduren fonksiyon iskeleti.

```rust
pub fn {{1|islem}}(girdi: &str) -> Result<String, Hata> {
    if girdi.is_empty() {
        return Err(Hata::BosGirdi);
    }
    Ok(girdi.to_uppercase())
}
```

## `Rust iterator zinciri`

- tetik: `;rust-iter`
- dil: `rust`
- etiket: `rust, iterasyon`
- kullanim: `0`
- dogrulama: `gecerli`

> filtre + map + collect zinciri.

```rust
{{1|kayitlar}}
    .iter()
    .filter(|k| {{2|kosul}})
    .map(|k| k.{{3|alan}})
    .collect::<Vec<_>>()
```

## `Rust match guard`

- tetik: `;rust-match`
- dil: `rust`
- etiket: `rust, match, iskelet`
- kullanim: `0`
- dogrulama: `gecerli`

> Guard kosulu ve varsayilan kollari olan match iskeleti.

```rust
match deger {
    Some(x) if x > {{1|esik|0}} => {{2|buyuk}},
    Some(_) => {{3|kucuk}},
    None => {{4|yok}},
}
```

## `Rust struct tanimi`

- tetik: `;rust-struct`
- dil: `rust`
- etiket: `rust, veri, iskelet`
- kullanim: `0`
- dogrulama: `gecerli`

> Alanlari ve turtilmis ozellikleri olan Rust struct iskeleti.

```rust
/// {{1|aciklama}}
#[derive(Debug, Clone, PartialEq)]
pub struct {{2|Kayit}} {
    pub {{3|ad}}: String,
}

impl {{2|Kayit}} {
    pub fn yeni({{3|ad}}: impl Into<String>) -> Self {
        Self { {{3|ad}}: {{3|ad}}.into() }
    }
}
```

## `Rust test fonksiyonu`

- tetik: `;rust-test`
- dil: `rust`
- etiket: `rust, test, iskelet`
- kullanim: `0`
- dogrulama: `gecerli`

> Isim yer tutucusuyla birim test iskeleti.

```rust
#[test]
fn {{1|adli}}_calisiyor() {
    let sonuc = {{2|ifade}};
    assert!(sonuc);
}
```

## `SQL guncelleme blogu`

- tetik: `;sql-guncel`
- dil: `sql`
- etiket: `sql, guncelleme`
- kullanim: `0`
- dogrulama: `denenmemis`

> Transaction icinde tek satir guncelleme.

```sql
BEGIN;
UPDATE {{1|tablo}} SET {{2|alan}} = {{3|deger}} WHERE id = {{4|kimlik}};
COMMIT;
```

## `SQL sorgu iskeleti`

- tetik: `;sql-sorgu`
- dil: `sql`
- etiket: `sql, sorgu, iskelet`
- kullanim: `1`
- dogrulama: `denenmemis`

> Konumsal ve varsayilan degerli yer tutucular tasiyan SQL iskeleti.

```sql
SELECT {{1|kolonlar}}
FROM {{2|tablo}}
WHERE {{3|kosul}}
ORDER BY {{4|siralama|olusturma}} DESC
LIMIT {{5|limit|100}};
```

## `SQL tarih filtresi`

- tetik: `;sql-tarih`
- dil: `sql`
- etiket: `sql, tarih, sorgu`
- kullanim: `0`
- dogrulama: `denenmemis`

> Tarih ve durum icin varsayilan degerli filtre.

```sql
SELECT * FROM {{1|tablo}}
WHERE olusturma >= '{{2|tarih|bugun}}'
  AND durum = '{{3|durum|aktif}}'
```

