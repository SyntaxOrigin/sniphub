//! Yapısal sözdizimi denetleyicisi — Rust ve Python.
//!
//! Kapsam: **gerçek dil grameri değildir.** Denetleyici yalnızca ayraç
//! dengesini, tırnak/escape kapanışını ve (Python için) girinti tutarlılığını
//! denetler. Tip hatası, ad çözümlemesi ve anlamsal kurallar kapsam dışıdır.
//! Bu, fikir raporundaki "doğrulandı" yerine **"yapısal denge kontrol edildi"**
//! ifadesini kullanma kararının gereğidir.
//!
//! `syn` ve `ruff` gibi hazır ayrıştırıcılar **kullanılmaz**:
//! WORKER_CONTRACT.md § 3.2-F bunları yasaklar ve gerçek gramer desteği
//! MANIFEST.md kartında ertelenmiştir.
//!
//! # Bilinen yanlış pozitif / negatifler
//!
//! - Rust'ta makro gövdeleri bağlama duyarlıdır; denetleyici onlara da
//!   yapısal kural uygular ve ayrıca `kapanmamis-makro` bulgusu üretir.
//! - `'a` biçimindeki yaşam süresi belirteci ile hatalı yazılmış karakter
//!   sabiti, geçerli Rust'ta bir yaşam süresinden sonra `;` **gelemeyeceği**
//!   gerçeğiyle ayrılır. Bu ayrım kusursuz değildir ancak gürültü üretmez.
//! - Python'da `TAB` sekizlik sütun adımıyla sayılır; karışık sekme/boşluk
//!   kullanımı denetlenmez.

use crate::dil::Dil;
use std::fmt;

/// Bir bulgunun ağırlığı.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ciddiyet {
    /// Yapı doğru, yalnızca dikkat gerektiriyor.
    Uyari,
    /// Yapısal bir ihlal bulundu.
    Hata,
}

impl fmt::Display for Ciddiyet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Ciddiyet::Uyari => "uyari",
            Ciddiyet::Hata => "hata",
        })
    }
}

/// Bulgunun nedeni.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BulguTuru {
    /// Kapanmamış `(`, `[` veya `{`.
    AcikAyrac,
    /// Eşleşmeyen kapanış ayracı.
    EslesmeyenKapanis,
    /// Kapanmamış dize, karakter sabiti veya blok yorum.
    KapanmamisTirnak,
    /// Eşleşmeyen girinti düzeyi.
    Girinti,
    /// Blok açılmış ama gövde girintilenmemiş.
    GirintisizGovde,
    /// Makro çağrısının gövdesinde kapanmamış ayraç.
    KapanmamisMakro,
    /// Denetleyicinin bu dil için kuralı yok.
    DilDesteklenmiyor,
}

impl fmt::Display for BulguTuru {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            BulguTuru::AcikAyrac => "acik-ayrac",
            BulguTuru::EslesmeyenKapanis => "eslesmeyen-kapanis",
            BulguTuru::KapanmamisTirnak => "kapanmamis-tirnak",
            BulguTuru::Girinti => "girinti",
            BulguTuru::GirintisizGovde => "girintisiz-govde",
            BulguTuru::KapanmamisMakro => "kapanmamis-makro",
            BulguTuru::DilDesteklenmiyor => "dil-desteklenmiyor",
        })
    }
}

/// Tek bir denetim bulgusu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bulgu {
    /// Ağırlık.
    pub ciddiyet: Ciddiyet,
    /// Neden.
    pub tur: BulguTuru,
    /// 1 tabanlı satır.
    pub satir: usize,
    /// 1 tabanlı sütun.
    pub sutun: usize,
    /// İnsan okuyan açıklama.
    pub mesaj: String,
}

impl fmt::Display for Bulgu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {} [{}] {}",
            self.satir, self.sutun, self.ciddiyet, self.tur, self.mesaj
        )
    }
}

/// Bir denetim turunun tamamı.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Denetim {
    /// Denetlenen dil.
    pub dil: Dil,
    /// Denetleyicinin bu dil için kuralı var mı.
    pub destekleniyor: bool,
    /// Bulunan sorunlar, kaynak sırasına göre.
    pub bulgular: Vec<Bulgu>,
}

impl Denetim {
    /// Yapısal denge kontrolünden geçtiyse `true`.
    pub fn gecerli(&self) -> bool {
        self.destekleniyor && self.bulgular.iter().all(|b| b.ciddiyet != Ciddiyet::Hata)
    }

    /// Yalnızca hata ağırlığındaki bulguları döndürür.
    pub fn hatalar(&self) -> Vec<&Bulgu> {
        self.bulgular
            .iter()
            .filter(|b| b.ciddiyet == Ciddiyet::Hata)
            .collect()
    }
}

/// Kod metnini verilen dil için yapısal olarak denetler.
///
/// Tanınmayan bir dil için `destekleniyor: false` döner ve yalnızca bir
/// [`BulguTuru::DilDesteklenmiyor`] uyarısı üretir; bu bir hata sayılmaz.
pub fn denetle(dil: Dil, kod: &str) -> Denetim {
    match dil {
        Dil::Rust => Denetim {
            dil,
            destekleniyor: true,
            bulgular: rust_tara(kod),
        },
        Dil::Python => Denetim {
            dil,
            destekleniyor: true,
            bulgular: python_tara(kod),
        },
        diger => Denetim {
            dil,
            destekleniyor: false,
            bulgular: vec![Bulgu {
                ciddiyet: Ciddiyet::Uyari,
                tur: BulguTuru::DilDesteklenmiyor,
                satir: 1,
                sutun: 1,
                mesaj: format!(
                    "{} için yapısal denge kuralı yok; yalnızca Rust ve Python denetlenir",
                    diger.gorunen_ad()
                ),
            }],
        },
    }
}

/// Ayraç türü.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ayrac {
    Parantez,
    Kose,
    Suslu,
}

impl Ayrac {
    fn acilis(self) -> char {
        match self {
            Ayrac::Parantez => '(',
            Ayrac::Kose => '[',
            Ayrac::Suslu => '{',
        }
    }

    fn kapanisi(self) -> char {
        match self {
            Ayrac::Parantez => ')',
            Ayrac::Kose => ']',
            Ayrac::Suslu => '}',
        }
    }

    fn ad(self) -> &'static str {
        match self {
            Ayrac::Parantez => "parantez",
            Ayrac::Kose => "köşeli parantez",
            Ayrac::Suslu => "süslü parantez",
        }
    }

    /// `(`, `[`, `{` karakterlerinden türü çözer.
    fn acilistan(ch: char) -> Option<Ayrac> {
        match ch {
            '(' => Some(Ayrac::Parantez),
            '[' => Some(Ayrac::Kose),
            '{' => Some(Ayrac::Suslu),
            _ => None,
        }
    }

    /// `)`, `]`, `}` karakterlerinden türü çözer.
    fn kapanisten(ch: char) -> Option<Ayrac> {
        match ch {
            ')' => Some(Ayrac::Parantez),
            ']' => Some(Ayrac::Kose),
            '}' => Some(Ayrac::Suslu),
            _ => None,
        }
    }
}

/// Yığın çerçevesi: açılan ayracın türü, kaynak konumu ve makro bağlamı.
#[derive(Debug, Clone, Copy)]
struct Cerceve {
    ayrac: Ayrac,
    satir: usize,
    sutun: usize,
    makroda: bool,
}

/// Karakter dizisi üzerinde ilerleyen ortak tarayıcı.
struct Tarayici {
    karakterler: Vec<char>,
    i: usize,
    satir: usize,
    sutun: usize,
}

impl Tarayici {
    fn yeni(kod: &str) -> Self {
        Tarayici {
            karakterler: kod.chars().collect(),
            i: 0,
            satir: 1,
            sutun: 1,
        }
    }

    fn bitti(&self) -> bool {
        self.i >= self.karakterler.len()
    }

    fn bak(&self, kac: usize) -> Option<char> {
        self.karakterler.get(self.i + kac).copied()
    }

    fn ilerle(&mut self) -> Option<char> {
        let ch = self.karakterler.get(self.i).copied()?;
        self.i += 1;
        if ch == '\n' {
            self.satir += 1;
            self.sutun = 1;
        } else {
            self.sutun += 1;
        }
        Some(ch)
    }

    fn atla(&mut self, adet: usize) {
        for _ in 0..adet {
            if self.ilerle().is_none() {
                break;
            }
        }
    }

    /// Tükettiğimiz karakterden hemen önceki karakter.
    fn onceki(&self) -> Option<char> {
        if self.i == 0 {
            None
        } else {
            self.karakterler.get(self.i - 1).copied()
        }
    }

    /// Satır başındaki boşluk/sekme sütunlarını sayar ve tüketir.
    fn girinti_olc(&mut self) -> usize {
        let mut sutun = 0usize;
        while let Some(ch) = self.bak(0) {
            match ch {
                ' ' => sutun += 1,
                '\t' => sutun = (sutun / 8 + 1) * 8,
                _ => break,
            }
            self.atla(1);
        }
        sutun
    }
}

/// Bulgu üretmeyen kısa yol.
fn bulgu(ciddiyet: Ciddiyet, tur: BulguTuru, satir: usize, sutun: usize, mesaj: String) -> Bulgu {
    Bulgu {
        ciddiyet,
        tur,
        satir,
        sutun,
        mesaj,
    }
}

/// Bir tırnak dizgesi gördüğünde alınan karar.
#[derive(Debug, Clone, Copy)]
enum TirnakKarari {
    /// N karakter tüket, sözcüksel durumu değiştirme.
    Atla(usize),
    /// N karakter tüket, normal dize durumuna geç.
    Dize(usize),
    /// N karakter tüket, ham dize durumuna geç (`#` sayısı ile).
    Ham(usize, usize),
    /// N karakter tüket, karakter sabiti durumuna geç.
    Karakter(usize),
}

/// Rust sözcüksel durumu (tüm alanları `Copy`, böylece eşleme sırasında ödünç alınmaz).
#[derive(Clone, Copy)]
enum RustDurum {
    Normal,
    SatirYorumu,
    BlokYorumu {
        derinlik: usize,
        satir: usize,
    },
    Dize {
        kacis: bool,
        satir: usize,
        sutun: usize,
    },
    HamDize {
        karma: usize,
        satir: usize,
        sutun: usize,
    },
    Karakter {
        satir: usize,
        sutun: usize,
    },
}

/// Rust kaynağını tarar.
fn rust_tara(kod: &str) -> Vec<Bulgu> {
    let mut t = Tarayici::yeni(kod);
    let mut durum = RustDurum::Normal;
    let mut yigin: Vec<Cerceve> = Vec::new();
    let mut bulgular: Vec<Bulgu> = Vec::new();
    // `ad!` görüldükten sonra gelen ilk ayraç makro gövdesidir.
    let mut makro_bekleniyor = false;

    while !t.bitti() {
        // --- Sözcüksel durum içinde ---
        match durum {
            RustDurum::SatirYorumu => {
                if t.ilerle() == Some('\n') {
                    durum = RustDurum::Normal;
                }
                continue;
            }
            RustDurum::BlokYorumu { derinlik, satir } => {
                if t.bak(0) == Some('/') && t.bak(1) == Some('*') {
                    durum = RustDurum::BlokYorumu {
                        derinlik: depth_up(derinlik),
                        satir,
                    };
                    t.atla(2);
                    continue;
                }
                if t.bak(0) == Some('*') && t.bak(1) == Some('/') {
                    if derinlik <= 1 {
                        durum = RustDurum::Normal;
                    } else {
                        durum = RustDurum::BlokYorumu {
                            derinlik: derinlik - 1,
                            satir,
                        };
                    }
                    t.atla(2);
                    continue;
                }
                t.ilerle();
                continue;
            }
            RustDurum::Dize {
                kacis,
                satir,
                sutun,
            } => {
                if kacis {
                    durum = RustDurum::Dize {
                        kacis: false,
                        satir,
                        sutun,
                    };
                    t.ilerle();
                    continue;
                }
                match t.bak(0) {
                    Some('\\') => {
                        durum = RustDurum::Dize {
                            kacis: true,
                            satir,
                            sutun,
                        };
                        t.ilerle();
                    }
                    Some('"') => {
                        durum = RustDurum::Normal;
                        t.ilerle();
                    }
                    _ => {
                        t.ilerle();
                    }
                }
                continue;
            }
            RustDurum::HamDize {
                karma,
                satir,
                sutun,
            } => {
                if t.bak(0) == Some('"') && (1..=karma).all(|k| t.bak(k) == Some('#')) {
                    durum = RustDurum::Normal;
                    t.atla(1 + karma);
                    continue;
                }
                t.ilerle();
                if t.bitti() {
                    bulgular.push(bulgu(
                        Ciddiyet::Hata,
                        BulguTuru::KapanmamisTirnak,
                        satir,
                        sutun,
                        format!(
                            "ham dizenin kapanışı bulunamadı (beklenen: \"{}{})",
                            "#".repeat(karma),
                            "#".repeat(karma)
                        ),
                    ));
                    return sirala(bulgular);
                }
                continue;
            }
            RustDurum::Karakter { satir, sutun } => {
                match t.bak(0) {
                    Some('\\') => t.atla(2),
                    Some('\'') => {
                        durum = RustDurum::Normal;
                        t.ilerle();
                    }
                    Some('\n') | None => {
                        bulgular.push(bulgu(
                            Ciddiyet::Hata,
                            BulguTuru::KapanmamisTirnak,
                            satir,
                            sutun,
                            "karakter sabiti satır sonunda kapatılmamış".into(),
                        ));
                        durum = RustDurum::Normal;
                        t.ilerle();
                    }
                    _ => {
                        t.ilerle();
                    }
                }
                continue;
            }
            RustDurum::Normal => {}
        }

        // --- Normal durum ---
        let ch = match t.bak(0) {
            Some(c) => c,
            None => break,
        };

        if ch == '/' && t.bak(1) == Some('/') {
            t.atla(2);
            durum = RustDurum::SatirYorumu;
            continue;
        }
        if ch == '/' && t.bak(1) == Some('*') {
            durum = RustDurum::BlokYorumu {
                derinlik: 1,
                satir: t.satir,
            };
            t.atla(2);
            continue;
        }
        if ch == '\'' {
            let (satir, sutun) = (t.satir, t.sutun);
            match tirnak_karari(&t) {
                Some(TirnakKarari::Atla(n)) => {
                    t.atla(n);
                }
                Some(TirnakKarari::Dize(n)) => {
                    t.atla(n);
                    durum = RustDurum::Dize {
                        kacis: false,
                        satir,
                        sutun,
                    };
                }
                Some(TirnakKarari::Ham(n, karma)) => {
                    t.atla(n);
                    durum = RustDurum::HamDize {
                        karma,
                        satir,
                        sutun,
                    };
                }
                Some(TirnakKarari::Karakter(n)) => {
                    t.atla(n);
                    durum = RustDurum::Karakter { satir, sutun };
                }
                None => {
                    t.ilerle();
                    durum = RustDurum::Karakter { satir, sutun };
                }
            }
            continue;
        }
        if ch == '"' {
            let (satir, sutun) = (t.satir, t.sutun);
            t.ilerle();
            durum = RustDurum::Dize {
                kacis: false,
                satir,
                sutun,
            };
            continue;
        }
        if let Some(karar) = onek_karari(&t) {
            let (satir, sutun) = (t.satir, t.sutun);
            match karar {
                TirnakKarari::Dize(n) => {
                    t.atla(n);
                    durum = RustDurum::Dize {
                        kacis: false,
                        satir,
                        sutun,
                    };
                }
                TirnakKarari::Ham(n, karma) => {
                    t.atla(n);
                    durum = RustDurum::HamDize {
                        karma,
                        satir,
                        sutun,
                    };
                }
                TirnakKarari::Karakter(n) => {
                    t.atla(n);
                    durum = RustDurum::Karakter { satir, sutun };
                }
                TirnakKarari::Atla(_) => {
                    t.ilerle();
                }
            }
            continue;
        }
        if let Some(tur) = Ayrac::acilistan(ch) {
            yigin.push(Cerceve {
                ayrac: tur,
                satir: t.satir,
                sutun: t.sutun,
                makroda: makro_bekleniyor,
            });
            makro_bekleniyor = false;
            t.ilerle();
            continue;
        }
        if let Some(tur) = Ayrac::kapanisten(ch) {
            let (satir, sutun) = (t.satir, t.sutun);
            match yigin.pop() {
                None => bulgular.push(bulgu(
                    Ciddiyet::Hata,
                    BulguTuru::EslesmeyenKapanis,
                    satir,
                    sutun,
                    format!("`{}` için açılış yok", tur.kapanisi()),
                )),
                Some(cerceve) if cerceve.ayrac != tur => bulgular.push(bulgu(
                    Ciddiyet::Hata,
                    BulguTuru::EslesmeyenKapanis,
                    satir,
                    sutun,
                    format!(
                        "`{}` kapanışı, satır {} sütun {}'de açılan `{}` ile eşleşmiyor",
                        tur.kapanisi(),
                        cerceve.satir,
                        cerceve.sutun,
                        cerceve.ayrac.acilis()
                    ),
                )),
                Some(_) => {}
            }
            makro_bekleniyor = false;
            t.ilerle();
            continue;
        }
        if ch == '!'
            && t.bak(1) != Some('=')
            && t.onceki().is_some_and(|c| c == '_' || c.is_alphanumeric())
        {
            makro_bekleniyor = true;
            t.ilerle();
            continue;
        }
        t.ilerle();
    }

    match durum {
        RustDurum::Dize { satir, sutun, .. } | RustDurum::Karakter { satir, sutun } => {
            bulgular.push(bulgu(
                Ciddiyet::Hata,
                BulguTuru::KapanmamisTirnak,
                satir,
                sutun,
                "dize veya karakter sabiti dosya sonunda kapatılmamış".into(),
            ));
        }
        RustDurum::BlokYorumu { derinlik, satir } if derinlik > 0 => {
            bulgular.push(bulgu(
                Ciddiyet::Hata,
                BulguTuru::KapanmamisTirnak,
                satir,
                1,
                "blok yorum kapatılmamış".into(),
            ));
        }
        _ => {}
    }

    for cerceve in yigin.iter().rev() {
        bulgular.push(bulgu(
            Ciddiyet::Hata,
            if cerceve.makroda {
                BulguTuru::KapanmamisMakro
            } else {
                BulguTuru::AcikAyrac
            },
            cerceve.satir,
            cerceve.sutun,
            format!(
                "{} `{}` açıldı ama kapatılmadı{}",
                cerceve.ayrac.ad(),
                cerceve.ayrac.acilis(),
                if cerceve.makroda {
                    " (makro çağrısı gövdesinde)"
                } else {
                    ""
                }
            ),
        ));
    }
    sirala(bulgular)
}

/// Blok yorum iç içe derinliğini bir artırır (taşma korumalı).
fn depth_up(derinlik: usize) -> usize {
    derinlik.saturating_add(1)
}

/// Bulguları satır/sütun sırasına dizer.
fn sirala(bulgular: Vec<Bulgu>) -> Vec<Bulgu> {
    let mut s = bulgular;
    s.sort_by_key(|b| (b.satir, b.sutun));
    s
}

/// `r`, `b` gibi öneklerle başlayan bir Rust dize/karakter sabiti çözümler.
fn onek_karari(t: &Tarayici) -> Option<TirnakKarari> {
    match t.bak(0)? {
        'r' | 'R' => {
            // `r"`, `r#"`, `r##"` ve `br"`, `br#"` varyantları
            let baslangic = if t.bak(1) == Some('b') || t.bak(1) == Some('B') {
                2
            } else {
                1
            };
            let mut j = baslangic;
            let mut karma = 0usize;
            while t.bak(j) == Some('#') {
                karma += 1;
                j += 1;
            }
            if t.bak(j) == Some('"') {
                Some(TirnakKarari::Ham(j + 1, karma))
            } else {
                None
            }
        }
        'b' | 'B' => match t.bak(1) {
            Some('"') => Some(TirnakKarari::Dize(2)),
            Some('\'') => Some(TirnakKarari::Karakter(2)),
            _ => None,
        },
        _ => None,
    }
}

/// Bir yaşam süresi belirtecinden sonra gelebilecek geçerli karakterler.
///
/// `;` bilinçli olarak **dışarıda** bırakıldı: geçerli Rust'ta bir yaşam
/// süresini `;` izleyemez, dolayısıyla bu konum hatalı yazılmış bir karakter
/// sabiti olarak yorumlanır.
fn yasam_suresi_sonrasi_gecerli(ch: Option<char>) -> bool {
    matches!(
        ch,
        Some('>')
            | Some(',')
            | Some('+')
            | Some(' ')
            | Some('&')
            | Some(')')
            | Some(':')
            | Some('(')
            | Some('=')
            | Some('{')
            | Some('\n')
    )
}

/// `'` gördüğünde karakter sabiti mi, yaşam süresi mi olduğuna karar verir.
fn tirnak_karari(t: &Tarayici) -> Option<TirnakKarari> {
    match t.bak(1) {
        Some('\\') => Some(TirnakKarari::Karakter(1)),
        Some(c) if c.is_alphabetic() || c == '_' => {
            let mut j = 1usize;
            while matches!(t.bak(j), Some(x) if x.is_alphanumeric() || x == '_') {
                j += 1;
            }
            if t.bak(j) == Some('\'') {
                // `'a'` — karakter sabiti
                return Some(TirnakKarari::Karakter(1));
            }
            if yasam_suresi_sonrasi_gecerli(t.bak(j)) {
                // `'a` — yaşam süresi belirteci, metnin parçası
                return Some(TirnakKarari::Atla(j));
            }
            None
        }
        _ => None,
    }
}

/// Python sözcüksel durumu (tüm alanları `Copy`).
#[derive(Clone, Copy)]
enum PyDurum {
    Normal,
    SatirYorumu,
    Dize {
        tirnak: char,
        uc: bool,
        ham: bool,
        kacis: bool,
    },
}

/// Python kaynağını tarar.
fn python_tara(kod: &str) -> Vec<Bulgu> {
    let mut t = Tarayici::yeni(kod);
    let mut durum = PyDurum::Normal;
    let mut yigin: Vec<Cerceve> = Vec::new();
    let mut bulgular: Vec<Bulgu> = Vec::new();
    let mut girintiler: Vec<usize> = vec![0];
    // Önceki mantıksal satır `:` ile bitti mi?
    let mut blok_basligi = false;
    // Son tüketilen anlamlı karakter (yorumlar hariç).
    let mut son_anlamli: Option<char> = None;
    let mut satir_basi = true;

    while !t.bitti() {
        match durum {
            PyDurum::SatirYorumu => {
                if t.ilerle() == Some('\n') {
                    durum = PyDurum::Normal;
                    satir_basi = true;
                }
                continue;
            }
            PyDurum::Dize {
                tirnak,
                uc,
                ham,
                kacis,
            } => {
                if kacis {
                    durum = PyDurum::Dize {
                        tirnak,
                        uc,
                        ham,
                        kacis: false,
                    };
                    t.ilerle();
                    continue;
                }
                if !ham && t.bak(0) == Some('\\') {
                    durum = PyDurum::Dize {
                        tirnak,
                        uc,
                        ham,
                        kacis: true,
                    };
                    t.ilerle();
                    continue;
                }
                if t.bak(0) == Some(tirnak) {
                    if uc {
                        if t.bak(1) == Some(tirnak) && t.bak(2) == Some(tirnak) {
                            durum = PyDurum::Normal;
                            t.atla(3);
                            continue;
                        }
                    } else {
                        durum = PyDurum::Normal;
                        t.ilerle();
                        continue;
                    }
                }
                t.ilerle();
                continue;
            }
            PyDurum::Normal => {}
        }

        // --- Normal durum ---
        if satir_basi {
            satir_basi = false;
            let girinti = t.girinti_olc();
            if t.bitti() {
                break;
            }
            let ilk = t.bak(0).unwrap_or('\n');
            if ilk == '\n' {
                t.ilerle();
                satir_basi = true;
                continue;
            }
            if ilk == '#' {
                durum = PyDurum::SatirYorumu;
                t.ilerle();
                continue;
            }
            if yigin.is_empty() {
                if blok_basligi && girinti <= *girintiler.last().unwrap_or(&0) {
                    bulgular.push(bulgu(
                        Ciddiyet::Hata,
                        BulguTuru::GirintisizGovde,
                        t.satir,
                        1,
                        "satır `:` ile bir blok açıyor ama gövde girintilenmemiş".into(),
                    ));
                }
                blok_basligi = false;
                let tepe = *girintiler.last().unwrap_or(&0);
                if girinti > tepe {
                    girintiler.push(girinti);
                } else {
                    while *girintiler.last().unwrap_or(&0) > girinti {
                        girintiler.pop();
                    }
                    if *girintiler.last().unwrap_or(&0) != girinti {
                        bulgular.push(bulgu(
                            Ciddiyet::Hata,
                            BulguTuru::Girinti,
                            t.satir,
                            1,
                            format!("girinti {girinti}, yığında eşleşen bir düzey bulamadı (tepe {tepe})"),
                        ));
                        girintiler.push(girinti);
                    }
                }
            }
            continue;
        }

        let ch = match t.bak(0) {
            Some(c) => c,
            None => break,
        };
        if ch == '\n' {
            if yigin.is_empty() {
                blok_basligi = son_anlamli == Some(':');
            }
            t.ilerle();
            satir_basi = true;
            continue;
        }
        if ch == '#' {
            durum = PyDurum::SatirYorumu;
            t.ilerle();
            continue;
        }
        if ch == '"' || ch == '\'' {
            let uc = t.bak(1) == Some(ch) && t.bak(2) == Some(ch);
            t.atla(if uc { 3 } else { 1 });
            durum = PyDurum::Dize {
                tirnak: ch,
                uc,
                ham: false,
                kacis: false,
            };
            son_anlamli = Some(ch);
            continue;
        }
        if ch == '\\' && t.bak(1) == Some('\n') {
            t.atla(2);
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            if let Some((tirnak, uc, ham, atlanacak)) = py_dize_onu(&t) {
                t.atla(atlanacak);
                durum = PyDurum::Dize {
                    tirnak,
                    uc,
                    ham,
                    kacis: false,
                };
                continue;
            }
            while matches!(t.bak(0), Some(c) if c.is_alphanumeric() || c == '_') {
                t.ilerle();
            }
            son_anlamli = Some(ch);
            continue;
        }
        if let Some(tur) = Ayrac::acilistan(ch) {
            yigin.push(Cerceve {
                ayrac: tur,
                satir: t.satir,
                sutun: t.sutun,
                makroda: false,
            });
            t.ilerle();
            son_anlamli = Some(ch);
            continue;
        }
        if let Some(tur) = Ayrac::kapanisten(ch) {
            let (satir, sutun) = (t.satir, t.sutun);
            match yigin.pop() {
                None => bulgular.push(bulgu(
                    Ciddiyet::Hata,
                    BulguTuru::EslesmeyenKapanis,
                    satir,
                    sutun,
                    format!("`{}` için açılış yok", tur.kapanisi()),
                )),
                Some(cerceve) if cerceve.ayrac != tur => bulgular.push(bulgu(
                    Ciddiyet::Hata,
                    BulguTuru::EslesmeyenKapanis,
                    satir,
                    sutun,
                    format!(
                        "`{}` kapanışı, satır {} sütun {}'de açılan `{}` ile eşleşmiyor",
                        tur.kapanisi(),
                        cerceve.satir,
                        cerceve.sutun,
                        cerceve.ayrac.acilis()
                    ),
                )),
                Some(_) => {}
            }
            t.ilerle();
            son_anlamli = Some(ch);
            continue;
        }
        t.ilerle();
        if !ch.is_whitespace() {
            son_anlamli = Some(ch);
        }
    }

    if let PyDurum::Dize { tirnak, uc, .. } = durum {
        bulgular.push(bulgu(
            Ciddiyet::Hata,
            BulguTuru::KapanmamisTirnak,
            t.satir,
            1,
            format!(
                "{} dizesi kapatılmamış{}",
                tirnak,
                if uc { " (üçlü tırnak)" } else { "" }
            ),
        ));
    }
    for cerceve in yigin.iter().rev() {
        bulgular.push(bulgu(
            Ciddiyet::Hata,
            BulguTuru::AcikAyrac,
            cerceve.satir,
            cerceve.sutun,
            format!("`{}` açıldı ama kapatılmadı", cerceve.ayrac.acilis()),
        ));
    }
    sirala(bulgular)
}

/// `r`/`b`/`f`/`u` öneklerinden bir Python dize başlangıcı çözümler.
///
/// Döner: `(tırnak karakteri, üçlü tırnak mı, ham mı, tüketilecek karakter)`.
fn py_dize_onu(t: &Tarayici) -> Option<(char, bool, bool, usize)> {
    let mut j = 0usize;
    let mut onek = String::new();
    while let Some(c) = t.bak(j) {
        if c.is_ascii_alphabetic() {
            onek.push(c.to_ascii_lowercase());
            j += 1;
        } else {
            break;
        }
    }
    if onek.is_empty() || !onek.chars().all(|c| "rbfu".contains(c)) {
        return None;
    }
    let tirnak = t.bak(j)?;
    if tirnak != '"' && tirnak != '\'' {
        return None;
    }
    let uc = t.bak(j + 1) == Some(tirnak) && t.bak(j + 2) == Some(tirnak);
    let ham = onek.contains('r');
    Some((tirnak, uc, ham, j + if uc { 3 } else { 1 }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turler(dil: Dil, kod: &str) -> Vec<BulguTuru> {
        denetle(dil, kod).bulgular.iter().map(|b| b.tur).collect()
    }

    #[test]
    fn dogru_rust_kodu_gecerlidir() {
        let d = denetle(
            Dil::Rust,
            "fn main() { let x = vec![1, 2, 3]; println!(\"{}\", x.len()); }",
        );
        assert!(d.gecerli(), "beklenmeyen bulgular: {:?}", d.bulgular);
    }

    #[test]
    fn rust_kapanmamis_parantez_bulunur() {
        let t = turler(Dil::Rust, "fn main() { foo(1, 2; }");
        assert!(t.contains(&BulguTuru::AcikAyrac), "gelen: {t:?}");
    }

    #[test]
    fn rust_kapanmamis_suslu_bulunur() {
        let t = turler(Dil::Rust, "fn main() { if x { ");
        assert!(t.contains(&BulguTuru::AcikAyrac), "gelen: {t:?}");
    }

    #[test]
    fn rust_kapanmamis_kose_bulunur() {
        let t = turler(Dil::Rust, "let v = [1, 2, 3;");
        assert!(t.contains(&BulguTuru::AcikAyrac), "gelen: {t:?}");
    }

    #[test]
    fn rust_eslesmeyen_kapanis_bulunur() {
        let t = turler(Dil::Rust, "let v = (1, 2];");
        assert!(t.contains(&BulguTuru::EslesmeyenKapanis), "gelen: {t:?}");
    }

    #[test]
    fn rust_tirnak_kacisi_islenir() {
        let d = denetle(Dil::Rust, "let s = \"a\\\"b(c\";");
        assert!(d.gecerli(), "bulgular: {:?}", d.bulgular);
    }

    #[test]
    fn rust_kapanmamis_tirnak_bulunur() {
        let t = turler(Dil::Rust, "let s = \"kapanmadi");
        assert!(t.contains(&BulguTuru::KapanmamisTirnak), "gelen: {t:?}");
    }

    #[test]
    fn rust_yasam_suresi_karakter_sabiti_sanilmaz() {
        let d = denetle(Dil::Rust, "fn f<'a, 'b>(x: &'a str) -> &'b str { x }");
        assert!(d.gecerli(), "bulgular: {:?}", d.bulgular);
    }

    #[test]
    fn rust_kapanmamis_karakter_sabiti_bulunur() {
        // Geçerli Rust'ta yaşam süresinden sonra `;` gelemez; bu yüzden
        // buradaki `'a` karakter sabiti olarak yorumlanır.
        let t = turler(Dil::Rust, "let c = 'a;");
        assert!(t.contains(&BulguTuru::KapanmamisTirnak), "gelen: {t:?}");
    }

    #[test]
    fn rust_kacisli_karakter_sabiti_gecerlidir() {
        assert!(denetle(Dil::Rust, "let c = '\\n';").gecerli());
        assert!(denetle(Dil::Rust, "let c = '\\'';").gecerli());
    }

    #[test]
    fn rust_makro_govdesinde_kapanmamis_ayrac_bulunur() {
        // Makro çağrısının gövdesinde kapanmamış ayraç ayrı bir bulgu olarak bildirilir.
        let t = turler(Dil::Rust, "fn main() { println!(\"x\", 1");
        assert!(t.contains(&BulguTuru::KapanmamisMakro), "gelen: {t:?}");
        // Kapanış ayracı yanlış türdeyse eşleşme hatası bildirilir.
        let t = turler(Dil::Rust, "fn main() { println!(\"x\", 1; }");
        assert!(t.contains(&BulguTuru::EslesmeyenKapanis), "gelen: {t:?}");
    }

    #[test]
    fn rust_makro_disi_ayrac_makro_sayilmaz() {
        // `!=` ve `if !x` makro çağrısı değildir.
        let t = turler(Dil::Rust, "fn f() { if a != b { foo(); }");
        assert!(!t.contains(&BulguTuru::KapanmamisMakro), "gelen: {t:?}");
    }

    #[test]
    fn rust_ham_dize_kapanisi_bulunur() {
        assert!(denetle(Dil::Rust, "let s = r\"ham ( metin\";").gecerli());
        assert!(denetle(Dil::Rust, "let s = r#\"a\"b\"#;").gecerli());
        // Kapanışta iki `#` var, açılışta üç: kapanış eşleşmez.
        let t = turler(Dil::Rust, "let s = r###\"a\"##;");
        assert!(t.contains(&BulguTuru::KapanmamisTirnak), "gelen: {t:?}");
    }

    #[test]
    fn rust_blok_yorum_kapanmamis_bulunur() {
        let t = turler(Dil::Rust, "/* yorum\nfn f() {}");
        assert!(t.contains(&BulguTuru::KapanmamisTirnak), "gelen: {t:?}");
    }

    #[test]
    fn dogru_python_kodu_gecerlidir() {
        let d = denetle(
            Dil::Python,
            "def f(x):\n    if x:\n        return 1\n    return 0\n",
        );
        assert!(d.gecerli(), "bulgular: {:?}", d.bulgular);
    }

    #[test]
    fn python_uc_tirnak_kapanmamis_bulunur() {
        let t = turler(Dil::Python, "s = \"\"\"acik\ndevam\n");
        assert!(t.contains(&BulguTuru::KapanmamisTirnak), "gelen: {t:?}");
    }

    #[test]
    fn python_uc_tirnak_kapanmis_gecerlidir() {
        let d = denetle(Dil::Python, "s = '''(parantez) icinde\ndeger'''\n");
        assert!(d.gecerli(), "bulgular: {:?}", d.bulgular);
    }

    #[test]
    fn python_ham_dizede_kacis_islenmez() {
        // Ham dizgede `\n` kaçış değildir: sondaki tırnak kapanışı sayılır.
        let d = denetle(Dil::Python, "s = r'\\n'\n");
        assert!(d.gecerli(), "bulgular: {:?}", d.bulgular);
        let t = turler(Dil::Python, "s = 'a\\'\n");
        assert!(t.contains(&BulguTuru::KapanmamisTirnak), "gelen: {t:?}");
    }

    #[test]
    fn python_girinti_hatasi_bulunur() {
        let t = turler(
            Dil::Python,
            "def f():\n    if x:\n        return 1\n      return 2\n",
        );
        assert!(t.contains(&BulguTuru::Girinti), "gelen: {t:?}");
    }

    #[test]
    fn python_girintisiz_govde_bulunur() {
        let t = turler(Dil::Python, "def f():\nreturn 1\n");
        assert!(t.contains(&BulguTuru::GirintisizGovde), "gelen: {t:?}");
    }

    #[test]
    fn python_parantez_icinde_girinti_denetlenmez() {
        let d = denetle(Dil::Python, "deger = topla(\n   1,\n     2,\n)\n");
        assert!(d.gecerli(), "bulgular: {:?}", d.bulgular);
    }

    #[test]
    fn python_yorum_icindeki_ayrac_goz_arazi() {
        let d = denetle(Dil::Python, "# yorum: )}\nx = 1\n");
        assert!(d.gecerli(), "bulgular: {:?}", d.bulgular);
    }

    #[test]
    fn desteklenmeyen_dil_uyari_verir() {
        let d = denetle(Dil::Sql, "SELECT * FROM t WHERE x IN (1, 2;");
        assert!(!d.destekleniyor);
        assert!(!d.gecerli());
        assert_eq!(d.bulgular.len(), 1);
        assert_eq!(d.bulgular[0].ciddiyet, Ciddiyet::Uyari);
    }

    #[test]
    fn gecerli_kod_hata_dondurmez() {
        assert!(denetle(Dil::Rust, "fn main() {}").hatalar().is_empty());
        assert!(denetle(Dil::Python, "x = 1\n").hatalar().is_empty());
    }

    #[test]
    fn bulgu_gosterimi_konumu_yazar() {
        let d = denetle(Dil::Rust, "let v = (1, 2;\nlet w = 3;\n");
        assert_eq!(
            d.bulgular.len(),
            1,
            "tek ve belirgin bulgu: {:?}",
            d.bulgular
        );
        let bulgu = d.bulgular.first().expect("bulgu");
        assert_eq!(bulgu.satir, 1);
        assert_eq!(bulgu.sutun, 9);
        assert!(bulgu.to_string().starts_with("1:9:"), "{}", bulgu);
    }
}
