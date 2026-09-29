# Builds the example snippet collection.
#
# Run from the project root:
#   powershell -NoProfile -ExecutionPolicy Bypass -File ornek\yukle-ornek.ps1
#
# The script regenerates `ornek\parcalar.json` from scratch, so it is safe to run
# more than once. Every snippet is added through `sniphub add`, which means the
# JSON file is the tool's own output rather than a hand-written file.
#
# Snippet bodies are written to temporary files and passed with `--body-file`.
# Reason: Windows PowerShell 5.1 mangles multi-line native-command arguments,
# and the console code page corrupts non-ASCII characters. Reading the body from
# a UTF-8 file avoids both problems.
$ErrorActionPreference = 'Stop'

$projeKoku = Split-Path -Parent $PSScriptRoot
$bin = Join-Path $projeKoku 'target\release\sniphub.exe'
$depo = Join-Path $PSScriptRoot 'parcalar.json'
$gvd = Join-Path $PSScriptRoot 'govdeler'
$utf8 = New-Object System.Text.UTF8Encoding $false

if (-not (Test-Path -LiteralPath $bin)) {
    throw "Binary not found. Run 'cargo build --release' first: $bin"
}

if (Test-Path -LiteralPath $depo) { Remove-Item -LiteralPath $depo -Force }
if (Test-Path -LiteralPath $gvd) { Remove-Item -LiteralPath $gvd -Recurse -Force }
New-Item -ItemType Directory -Path $gvd | Out-Null

# Writes a snippet body to a UTF-8 file and returns its path.
function Govde([string] $ad, [string] $icerik) {
    $yol = Join-Path $gvd ($ad + '.txt')
    [System.IO.File]::WriteAllText($yol, $icerik, $utf8)
    return $yol
}

function Ekle([string] $ad, [string] $tetik, [string] $dil, [string] $aciklama, [string] $etiketler, [string] $govde) {
    & $bin --depo $depo add --name $ad --trigger $tetik --lang $dil --desc $aciklama --tag $etiketler --body-file $govde | Out-Null
}

# --- Rust ---------------------------------------------------------------------

Ekle 'Rust struct tanimi' ';rust-struct' 'rust' 'Alanlari ve turtilmis ozellikleri olan Rust struct iskeleti.' 'rust,veri,iskelet' (Govde 'rust-struct' @'
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
'@)

Ekle 'Rust dosya okuma' ';rust-dosya' 'rust' 'Argumandan yol alip dosyayi String olarak okuyan fonksiyon.' 'rust,dosya,io' (Govde 'rust-dosya' @'
fn {{1|okuma|oku}}() -> std::io::Result<String> {
    let yol = std::env::args().nth(1).unwrap_or_else(|| "veri.txt".to_string());
    Ok(std::fs::read_to_string(&yol)?)
}
'@)

Ekle 'Rust hata yakalama' ';rust-hata' 'rust' 'Result tabanli hata donduren fonksiyon iskeleti.' 'rust,hata,iskelet' (Govde 'rust-hata' @'
pub fn {{1|islem}}(girdi: &str) -> Result<String, Hata> {
    if girdi.is_empty() {
        return Err(Hata::BosGirdi);
    }
    Ok(girdi.to_uppercase())
}
'@)

Ekle 'Rust test fonksiyonu' ';rust-test' 'rust' 'Isim yer tutucusuyla birim test iskeleti.' 'rust,test,iskelet' (Govde 'rust-test' @'
#[test]
fn {{1|adli}}_calisiyor() {
    let sonuc = {{2|ifade}};
    assert!(sonuc);
}
'@)

Ekle 'Rust iterator zinciri' ';rust-iter' 'rust' 'filtre + map + collect zinciri.' 'rust,iterasyon' (Govde 'rust-iter' @'
{{1|kayitlar}}
    .iter()
    .filter(|k| {{2|kosul}})
    .map(|k| k.{{3|alan}})
    .collect::<Vec<_>>()
'@)

Ekle 'Rust match guard' ';rust-match' 'rust' 'Guard kosulu ve varsayilan kollari olan match iskeleti.' 'rust,match,iskelet' (Govde 'rust-match' @'
match deger {
    Some(x) if x > {{1|esik|0}} => {{2|buyuk}},
    Some(_) => {{3|kucuk}},
    None => {{4|yok}},
}
'@)

Ekle 'Rust Display uygulamasi' ';rust-display' 'rust' 'Display trait uygulamasi; imlec konumu isaretlidir.' 'rust,trait,fmt' (Govde 'rust-display' @'
impl std::fmt::Display for {{1|Tip}} {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{2|okuma}}", self.{{3|alan}}){{imlec}}
    }
}
'@)

# Deliberately broken: unclosed delimiter, so `check` has something to report.
Ekle 'Rust bozuk ornek' ';rust-bozuk' 'rust' 'Yapisal denge kontrolunun yakalamasi icin bilerek kapanmamis ayrac.' 'rust,test,bozuk' (Govde 'rust-bozuk' @'
fn bozuk() {
    let v = vec![1, 2, 3;
}
'@)

# --- Python -------------------------------------------------------------------

Ekle 'Python dosya okuma' ';py-oku' 'python' 'with deyimi ve ucle tirnakli dokustrum iceren fonksiyon.' 'python,dosya,okuma' (Govde 'py-oku' @'
def {{1|oku}}(yol: str) -> str:
    """{{2|dosya}} dosyasını metin olarak okur."""
    with open(yol, encoding="utf-8") as dosya:
        return dosya.read()
'@)

Ekle 'Python CSV okuyucu' ';py-csv' 'python' 'csv.DictReader ile sozluk listesine donusturen fonksiyon.' 'python,csv,veri' (Govde 'py-csv' @'
import csv

def {{1|yukle}}(yol: str) -> list[dict]:
    with open(yol, newline="", encoding="utf-8") as dosya:
        return list(csv.DictReader(dosya))
'@)

Ekle 'Python sozluk siralama' ';py-sirala' 'python' 'lambda ile cok satirlik sorted cagrisi.' 'python,siralama,lamba' (Govde 'py-sirala' @'
def {{1|sirala}}(kayitlar: list[dict]) -> list[dict]:
    return sorted(
        kayitlar,
        key=lambda k: k.get("{{2|alan}}", ""),
        reverse={{3|ters|False}},
    )
'@)

Ekle 'Python hata yakalama' ';py-hata' 'python' 'except blogu ve f-string mesaji.' 'python,hata' (Govde 'py-hata' @'
def {{1|calistir}}(is_levi):
    try:
        return is_levi()
    except {{2|TipHata}} as hata:
        print(f"hata: {hata}")
        raise
'@)

Ekle 'Python dataclass' ';py-dataclass' 'python' 'Varsayilan degerli dataclass tanimi.' 'python,veri,iskelet' (Govde 'py-dataclass' @'
from dataclasses import dataclass

@dataclass
class {{1|Kayit}}:
    {{2|ad}}: str
    {{3|deger}}: int = 0
'@)

Ekle 'Python liste donusumu' ';py-liste' 'python' 'Liste ve sozluk kapsayici anlayislari.' 'python,anlayis' (Govde 'py-liste' @'
liste = [{{1|eleman}} for oge in {{2|kaynak}} if {{3|kosul}}]
sozluk = {oge[0]: oge[1] for oge in {{2|kaynak}}}
'@)

# Deliberately broken indentation, so the indent checker has something to report.
Ekle 'Python bozuk girinti' ';py-bozuk' 'python' 'Girinti denetimini tetiklemek icin bilinen hatali girinti.' 'python,test,bozuk' (Govde 'py-bozuk' @'
def {{1|kontrol}}(x):
    if x > 0:
        return "pozitif"
      return "negatif"
'@)

# --- SQL (outside the checker's scope; present for search coverage) -----------

Ekle 'SQL sorgu iskeleti' ';sql-sorgu' 'sql' 'Konumsal ve varsayilan degerli yer tutucular tasiyan SQL iskeleti.' 'sql,sorgu,iskelet' (Govde 'sql-sorgu' @'
SELECT {{1|kolonlar}}
FROM {{2|tablo}}
WHERE {{3|kosul}}
ORDER BY {{4|siralama|olusturma}} DESC
LIMIT {{5|limit|100}};
'@)

Ekle 'SQL tarih filtresi' ';sql-tarih' 'sql' 'Tarih ve durum icin varsayilan degerli filtre.' 'sql,tarih,sorgu' (Govde 'sql-tarih' @'
SELECT * FROM {{1|tablo}}
WHERE olusturma >= '{{2|tarih|bugun}}'
  AND durum = '{{3|durum|aktif}}'
'@)

Ekle 'SQL guncelleme blogu' ';sql-guncel' 'sql' 'Transaction icinde tek satir guncelleme.' 'sql,guncelleme' (Govde 'sql-guncel' @'
BEGIN;
UPDATE {{1|tablo}} SET {{2|alan}} = {{3|deger}} WHERE id = {{4|kimlik}};
COMMIT;
'@)

Write-Host "example repo written: $depo"
& $bin --depo $depo stats --top 5
