//! Genişleme dili: `{{belirtec}}` ayrıştırma ve genişletme.
//!
//! Kapsam: parça gövdesindeki konum belirteçlerini çözümler ve genişletilmiş
//! metni **yer tutucu yuvalarıyla birlikte** döner. Sözdizimi denetleme,
//! depo yönetimi ve istatistik bu modülün sorumluluğunda değildir.
//!
//! # Söz dizimi
//!
//! Belirteçler `{{` … `}}` arasındadır. `\{{` ve `\}}` kaçışları literaldir.
//! İçerik `|` ile alanlara bölünür:
//!
//! | Biçim | Anlam |
//! |---|---|
//! | `{{imleç}}` | İmleç konumu; hiçbir metin üretmez |
//! | `{{secim}}` | Seçim bölgesi; kullanıcı seçimi ya da varsayılan metin |
//! | `{{1}}` | 1 tabanlı konumsal yer tutucu |
//! | `{{1\|limit}}` | Konumsal + adlı |
//! | `{{1\|limit\|100}}` | Konumsal + adlı + varsayılan |
//! | `{{limit}}` | Yalnız adlı |
//! | `{{limit\|100}}` | Adlı + varsayılan |
//! | `{{tarih}}` `{{saat}}` `{{zaman}}` `{{kullanici}}` | Yerleşikler |
//!
//! Varsayılan değerlerin içinde başka belirteçler bulunabilir
//! (`{{a\|{{tarih}}}}`); iç içe çözümleme [`ICICE_TAVANI`] ile sınırlıdır ve
//! toplam adım sayısı [`ADIM_TAVANI`] tavanına bağlıdır. Bu iki tavan,
//! "kötü niyetli paket sonsuz döngü yaratır" tehlikesine karşı zorunludur.

use crate::hata::{Hata, Sonuc};
use crate::zaman::An;
use std::collections::BTreeMap;
use std::fmt;

/// Bir genişleme adımında çözülebilecek en fazla belirteç sayısı.
pub const ADIM_TAVANI: usize = 32;

/// Varsayılan değer içinde izin verilen en derin iç içe çözümleme.
pub const ICICE_TAVANI: usize = 8;

/// Belirtecin türü.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BelirtecTuru {
    /// `{{imleç}}` — genişletilen metinde imlecin oturacağı konum.
    Imlec,
    /// `{{secim}}` — genişletilen metinde seçilecek bölge.
    Secim,
    /// `{{1}}` gibi konumsal yer tutucu.
    Konumsal,
    /// `{{ad}}` gibi adlı yer tutucu.
    Adli,
    /// `{{tarih}}` gibi yerleşik.
    Yerlesik,
}

impl BelirtecTuru {
    /// Kullanıcının doldurması gereken alan mı.
    pub const fn doldurulabilir(self) -> bool {
        matches!(self, BelirtecTuru::Konumsal | BelirtecTuru::Adli)
    }
}

/// Ayrıştırılmış bir belirteç.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Belirtec {
    /// Belirtecin türü.
    pub tur: BelirtecTuru,
    /// Konumsal belirteçlerin 1 tabanlı indeksi.
    pub indeks: Option<u32>,
    /// Adlı belirteçlerin / yerleşiklerin adı.
    pub ad: Option<String>,
    /// Varsa varsayılan değer ham metni.
    pub varsayilan: Option<String>,
    /// Kaynak metindeki 1 tabanlı satır.
    pub satir: usize,
    /// Kaynak metindeki 1 tabanlı sütun.
    pub sutun: usize,
}

impl fmt::Display for Belirtec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{{")?;
        if let Some(i) = self.indeks {
            write!(f, "{i}")?;
        }
        if let Some(a) = &self.ad {
            if self.indeks.is_some() {
                write!(f, "|{a}")?;
            } else {
                write!(f, "{a}")?;
            }
        }
        if let Some(v) = &self.varsayilan {
            write!(f, "|{v}")?;
        }
        write!(f, "}}}}")
    }
}

/// Genişletilmiş metindeki tek bir yer tutucu yuvası.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Yuva {
    /// Metinde görünme sırası (1 tabanlı).
    pub sira: usize,
    /// Belirtecin türü.
    pub tur: BelirtecTuru,
    /// Adı varsa.
    pub ad: Option<String>,
    /// Gösterimde kullanılacak kısa etiket (yoksa `{{n}}`).
    pub etiket: String,
    /// Yuvaya yazılan nihai metin.
    pub deger: String,
    /// Kullanıcı değeri verdiyse `true`.
    pub dolduruldu: bool,
    /// Genişletilmiş metindeki bayt ofseti (başlangıç, bitiş).
    pub baslangic: usize,
    /// Genişletilmiş metindeki bayt ofseti (bitiş).
    pub bitis: usize,
    /// Genişletilmiş metindeki 1 tabanlı satır.
    pub satir: usize,
    /// Genişletilmiş metindeki 1 tabanlı sütun.
    pub sutun: usize,
}

/// Bir genişleme sonucunun tamamı.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Genisleme {
    /// Yer tutucular çözüldükten sonra kalan metin.
    pub metin: String,
    /// Çözülen yuvalar, görünme sırasına göre.
    pub yuvalar: Vec<Yuva>,
}

impl Genisleme {
    /// `{{imleç}}` belirtecinin genişletilmiş metindeki bayt ofseti.
    pub fn imlec_konumu(&self) -> Option<usize> {
        self.yuvalar
            .iter()
            .find(|y| y.tur == BelirtecTuru::Imlec)
            .map(|y| y.baslangic)
    }

    /// `{{secim}}` bölgesinin metin içindeki aralığı.
    pub fn secim_araligi(&self) -> Option<(usize, usize)> {
        self.yuvalar
            .iter()
            .find(|y| y.tur == BelirtecTuru::Secim)
            .map(|y| (y.baslangic, y.bitis))
    }

    /// Doldurulması gereken yuvaların, metin içindeki konum sırasına göre
    /// dizilmiş hâli (fikir raporu § 03, senaryo S2).
    pub fn odak_sirasi(&self) -> Vec<&Yuva> {
        let mut sirali: Vec<&Yuva> = self
            .yuvalar
            .iter()
            .filter(|y| y.tur.doldurulabilir())
            .collect();
        sirali.sort_by_key(|y| y.baslangic);
        sirali
    }
}

/// Genişleme ortamı: kullanıcı değerleri, zaman görüntüsü ve kullanıcı adı.
#[derive(Debug, Clone)]
pub struct Genisletici {
    degerler: BTreeMap<String, String>,
    an: An,
    kullanici: String,
}

impl Genisletici {
    /// Sistem saatinden ve ortam değişkenlerinden değer üreten genişletici.
    pub fn varsayilan() -> Self {
        let kullanici = ["USER", "USERNAME", "LOGNAME"]
            .iter()
            .find_map(|k| std::env::var(k).ok())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "kullanici".to_string());
        Genisletici {
            degerler: BTreeMap::new(),
            an: An::simdi(),
            kullanici,
        }
    }

    /// Testlerin ve tekrarlanabilir çıktıların kullanacağı sabit genişletici.
    pub fn sabit(an: An, kullanici: &str) -> Self {
        Genisletici {
            degerler: BTreeMap::new(),
            an,
            kullanici: kullanici.to_string(),
        }
    }

    /// `anahtar=değer` biçimindeki tek bir atamayı ekler.
    pub fn deger_ekle(&mut self, atama: &str) -> Sonuc<()> {
        let (anahtar, deger) = atama.split_once('=').ok_or_else(|| Hata::GecersizDeger {
            ham: atama.to_string(),
        })?;
        let anahtar = anahtar.trim();
        if anahtar.is_empty() {
            return Err(Hata::GecersizDeger {
                ham: atama.to_string(),
            });
        }
        self.degerler.insert(anahtar.to_string(), deger.to_string());
        Ok(())
    }

    /// Gövdeyi genişletir.
    pub fn genislet(&self, govde: &str) -> Sonuc<Genisleme> {
        let mut cikti = String::with_capacity(govde.len() + 16);
        let mut yuvalar: Vec<Yuva> = Vec::new();
        let mut adim = 0usize;
        self.yaz(govde, &mut cikti, &mut yuvalar, &mut adim, 0)?;
        for (i, yuva) in yuvalar.iter_mut().enumerate() {
            yuva.sira = i + 1;
            let (satir, sutun) = konum(&cikti, yuva.baslangic);
            yuva.satir = satir;
            yuva.sutun = sutun;
        }
        Ok(Genisleme {
            metin: cikti,
            yuvalar,
        })
    }

    /// Ana ayrıştırma/genişletme döngüsü.
    fn yaz(
        &self,
        kaynak: &str,
        cikti: &mut String,
        yuvalar: &mut Vec<Yuva>,
        adim: &mut usize,
        derinlik: usize,
    ) -> Sonuc<()> {
        if derinlik > ICICE_TAVANI {
            return Err(Hata::AdimTavani {
                tavan: ICICE_TAVANI,
            });
        }
        let baytlar = kaynak.as_bytes();
        let n = baytlar.len();
        let mut i = 0usize;
        while i < n {
            if baytlar[i] == b'\\'
                && i + 1 < n
                && (baytlar[i + 1] == b'{' || baytlar[i + 1] == b'}')
            {
                cikti.push(char::from(baytlar[i + 1]));
                i += 2;
                continue;
            }
            if baytlar[i] == b'}' && i + 1 < n && baytlar[i + 1] == b'}' {
                let (satir, sutun) = konum(kaynak, i);
                return Err(Hata::AsiriKapanis { satir, sutun });
            }
            if baytlar[i] == b'{' && i + 1 < n && baytlar[i + 1] == b'{' {
                let (satir, sutun) = konum(kaynak, i);
                let bitis =
                    bul_kapat(baytlar, i + 2).ok_or(Hata::KapanmamisBelirtec { satir, sutun })?;
                let govde = &kaynak[i + 2..bitis];
                let belirtec = ayikla(govde, satir, sutun)?;
                *adim += 1;
                if *adim > ADIM_TAVANI {
                    return Err(Hata::AdimTavani { tavan: ADIM_TAVANI });
                }
                self.belirtec_ciz(&belirtec, cikti, yuvalar, adim, derinlik)?;
                i = bitis + 2;
                continue;
            }
            let ch = kaynak[i..].chars().next().unwrap_or('\u{0}');
            cikti.push(ch);
            i += ch.len_utf8();
        }
        Ok(())
    }

    /// Ayrıştırılmış tek bir belirteci metne yazar.
    ///
    /// Konum alanları yalnızca *genişletilmiş* metin için anlamlıdır ve
    /// `genislet` içinde doldurulur; bu yüzden burada taşınmaz.
    fn belirtec_ciz(
        &self,
        belirtec: &Belirtec,
        cikti: &mut String,
        yuvalar: &mut Vec<Yuva>,
        adim: &mut usize,
        derinlik: usize,
    ) -> Sonuc<()> {
        let baslangic = cikti.len();

        // Çözümleme sırası: kullanıcı değeri > varsayılan > boş metin.
        let (tur, etiket, mut deger, dolduruldu) = match belirtec.tur {
            BelirtecTuru::Imlec => (
                BelirtecTuru::Imlec,
                "imleç".to_string(),
                String::new(),
                false,
            ),
            BelirtecTuru::Secim => {
                let verilen = self.degerler.get("secim").cloned();
                let d = verilen
                    .clone()
                    .or_else(|| belirtec.varsayilan.clone())
                    .unwrap_or_default();
                (
                    BelirtecTuru::Secim,
                    "seçim".to_string(),
                    d,
                    verilen.is_some(),
                )
            }
            BelirtecTuru::Yerlesik => {
                let ad = belirtec.ad.clone().unwrap_or_default();
                let d = self.yerlesik(&ad);
                (BelirtecTuru::Yerlesik, ad.clone(), d, true)
            }
            BelirtecTuru::Konumsal => {
                let indeks = belirtec.indeks.unwrap_or(0);
                let verilen = self.degerler.get(&indeks.to_string()).cloned().or_else(|| {
                    belirtec
                        .ad
                        .as_ref()
                        .and_then(|a| self.degerler.get(a))
                        .cloned()
                });
                let etiket = belirtec
                    .ad
                    .clone()
                    .unwrap_or_else(|| format!("{{{indeks}}}"));
                let d = verilen
                    .clone()
                    .or_else(|| belirtec.varsayilan.clone())
                    .unwrap_or_default();
                (BelirtecTuru::Konumsal, etiket, d, verilen.is_some())
            }
            BelirtecTuru::Adli => {
                let ad = belirtec.ad.clone().unwrap_or_default();
                let verilen = self.degerler.get(&ad).cloned();
                let d = verilen
                    .clone()
                    .or_else(|| belirtec.varsayilan.clone())
                    .unwrap_or_default();
                (BelirtecTuru::Adli, ad.clone(), d, verilen.is_some())
            }
        };

        // Varsayılan değerin kendi içinde belirteç barındırabilir.
        if tur != BelirtecTuru::Yerlesik && (deger.contains("{{") || deger.contains("\\{{")) {
            let mut ic = String::new();
            self.yaz(&deger, &mut ic, &mut Vec::new(), adim, derinlik + 1)?;
            deger = ic;
        }

        cikti.push_str(&deger);
        let bitis = cikti.len();
        if tur != BelirtecTuru::Yerlesik {
            yuvalar.push(Yuva {
                sira: 0,
                tur,
                ad: belirtec.ad.clone(),
                etiket,
                deger,
                dolduruldu,
                baslangic,
                bitis,
                satir: 1,
                sutun: 1,
            });
        }
        Ok(())
    }

    /// Yerleşik belirteçlerin değerini döner.
    fn yerlesik(&self, ad: &str) -> String {
        match ad {
            "tarih" => self.an.tarih.clone(),
            "saat" => self.an.saat.clone(),
            "zaman" => self.an.zaman.clone(),
            "kullanici" => self.kullanici.clone(),
            _ => String::new(),
        }
    }
}

/// Ayrıştırılmış belirteçten üretilebilen tuş sayacı için ham uzunluk.
fn ayikla(govde: &str, satir: usize, sutun: usize) -> Sonuc<Belirtec> {
    let alanlar: Vec<&str> = govde.split('|').map(str::trim).collect();
    if alanlar.len() > 3 {
        return Err(Hata::GecersizBelirtec {
            metin: govde.to_string(),
            satir,
            sutun,
            neden: "en fazla üç alan destekleniyor (indeks|ad|varsayılan)",
        });
    }
    if alanlar.iter().all(|a| a.is_empty()) {
        return Err(Hata::GecersizBelirtec {
            metin: govde.to_string(),
            satir,
            sutun,
            neden: "belirteç boş",
        });
    }

    let ilk = alanlar[0];
    let sayisal_mi = !ilk.is_empty() && ilk.chars().all(|c| c.is_ascii_digit());

    if sayisal_mi {
        let indeks: u32 = ilk.parse().map_err(|_| Hata::GecersizBelirtec {
            metin: govde.to_string(),
            satir,
            sutun,
            neden: "konumsal indeks okunamadı",
        })?;
        if indeks == 0 {
            return Err(Hata::GecersizBelirtec {
                metin: govde.to_string(),
                satir,
                sutun,
                neden: "konumsal indeks 1'den başlar",
            });
        }
        let ad = alanlar.get(1).copied().filter(|a| !a.is_empty());
        if let Some(a) = ad {
            if !gecerli_ad(a) {
                return Err(Hata::GecersizBelirtec {
                    metin: govde.to_string(),
                    satir,
                    sutun,
                    neden: "ad harf veya alt çizgi ile başlamalı",
                });
            }
        }
        let varsayilan = alanlar.get(2).copied().filter(|a| !a.is_empty());
        return Ok(Belirtec {
            tur: BelirtecTuru::Konumsal,
            indeks: Some(indeks),
            ad: ad.map(str::to_string),
            varsayilan: varsayilan.map(str::to_string),
            satir,
            sutun,
        });
    }

    let ad = ilk;
    // Yerleşik adlar ASCII'ye çevrilmeden önce tanınır: `{{imleç}}` ve
    // `{{saat}}` gibi adlar harf doğrulamasını geçmez.
    if let Some(tur) = yerlesik_turu(ilk) {
        if tur == BelirtecTuru::Yerlesik && varsayilan_var(&alanlar) {
            return Err(Hata::GecersizBelirtec {
                metin: govde.to_string(),
                satir,
                sutun,
                neden: "yerleşikler varsayılan değer almaz",
            });
        }
        return Ok(Belirtec {
            tur,
            indeks: None,
            ad: Some(ilk.to_string()),
            varsayilan: alanlar
                .get(1)
                .copied()
                .filter(|a| !a.is_empty())
                .map(str::to_string),
            satir,
            sutun,
        });
    }

    if !gecerli_ad(ad) {
        return Err(Hata::GecersizBelirtec {
            metin: govde.to_string(),
            satir,
            sutun,
            neden: "bilinmeyen belirteç; ad harf veya alt çizgi ile başlamalı",
        });
    }
    let varsayilan = alanlar.get(1).copied().filter(|a| !a.is_empty());
    Ok(Belirtec {
        tur: BelirtecTuru::Adli,
        indeks: None,
        ad: Some(ad.to_string()),
        varsayilan: varsayilan.map(str::to_string),
        satir,
        sutun,
    })
}

/// İkinci alan dolu mu.
fn varsayilan_var(alanlar: &[&str]) -> bool {
    alanlar.get(1).is_some_and(|a| !a.is_empty())
}

/// Yerleşik adı tanınan türü döner.
fn yerlesik_turu(ad: &str) -> Option<BelirtecTuru> {
    match ad {
        "imleç" => Some(BelirtecTuru::Imlec),
        "secim" => Some(BelirtecTuru::Secim),
        "tarih" | "saat" | "zaman" | "kullanici" => Some(BelirtecTuru::Yerlesik),
        _ => None,
    }
}

/// `{{` açılışının eşleşen `}}` kapanışının bayt ofsetini bulur.
///
/// İç içe belirteçleri doğru saymak için derinlik takibi yapar; kaçışlı
/// `\{{` ve `\}}` dizileri sayıma katılmaz.
fn bul_kapat(baytlar: &[u8], baslangic: usize) -> Option<usize> {
    let n = baytlar.len();
    let mut i = baslangic;
    let mut derinlik = 1usize;
    while i < n {
        if baytlar[i] == b'\\' && i + 1 < n && (baytlar[i + 1] == b'{' || baytlar[i + 1] == b'}') {
            i += 2;
            continue;
        }
        if baytlar[i] == b'{' && i + 1 < n && baytlar[i + 1] == b'{' {
            derinlik += 1;
            i += 2;
            continue;
        }
        if baytlar[i] == b'}' && i + 1 < n && baytlar[i + 1] == b'}' {
            derinlik -= 1;
            if derinlik == 0 {
                return Some(i);
            }
            i += 2;
            continue;
        }
        i += 1;
    }
    None
}

/// Bir bayt ofsetinin metindeki 1 tabanlı satır/sütun konumunu döner.
fn konum(metin: &str, ofset: usize) -> (usize, usize) {
    let ofset = ofset.min(metin.len());
    let on = &metin[..ofset];
    let satir = on.matches('\n').count() + 1;
    let sutun = match on.rfind('\n') {
        Some(i) => on[i + 1..].chars().count() + 1,
        None => on.chars().count() + 1,
    };
    (satir, sutun)
}

/// Bir yer tutucu adının geçerli olup olmadığını denetler.
pub fn gecerli_ad(ad: &str) -> bool {
    let mut karakterler = ad.chars();
    let ilk_gecerli = match karakterler.next() {
        None => false,
        Some(ilk) => ilk.is_ascii_alphabetic() || ilk == '_',
    };
    ilk_gecerli && karakterler.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_genisletici() -> Genisletici {
        Genisletici::sabit(An::epoch_saniye(1_790_685_296), "deniz")
    }

    fn genislet(govde: &str) -> Genisleme {
        test_genisletici().genislet(govde).expect("genisleme")
    }

    #[test]
    fn imlec_belirteci_metin_uretmez() {
        let g = genislet("fn main() { {{imleç}} }");
        assert_eq!(g.metin, "fn main() {  }");
        assert_eq!(g.imlec_konumu(), Some(12));
    }

    #[test]
    fn secim_belirteci_aralik_uretir() {
        let g = genislet("{{secim}};");
        assert_eq!(g.metin, ";");
        assert_eq!(g.secim_araligi(), Some((0, 0)));
    }

    #[test]
    fn konumsal_belirtece_deger_yazilir() {
        let mut g = test_genisletici();
        g.deger_ekle("1=public").expect("atama");
        g.deger_ekle("2=kayitlar").expect("atama");
        let sonuc = g.genislet("{{1}}.{{2}}").expect("genisleme");
        assert_eq!(sonuc.metin, "public.kayitlar");
    }

    #[test]
    fn varsayilanli_konumsal_belirtec_calisir() {
        let g = genislet("LIMIT {{3|limit|100}};");
        assert_eq!(g.metin, "LIMIT 100;");
        assert_eq!(g.yuvalar[0].etiket, "limit");
        assert!(!g.yuvalar[0].dolduruldu);
    }

    #[test]
    fn adli_belirtec_varsayilan_alir() {
        let g = genislet("WHERE x >= {{tarih_kayit|bugun}}");
        assert_eq!(g.metin, "WHERE x >= bugun");
    }

    #[test]
    fn yerlesikler_sabit_zamandan_dogur() {
        let g = genislet("{{tarih}} {{saat}} {{zaman}} {{kullanici}}");
        assert_eq!(g.metin, "2026-09-29 12:34:56 2026-09-29T12:34:56Z deniz");
    }

    #[test]
    fn ic_ice_belirtec_cozulur() {
        let g = genislet("{{1|ts|{{tarih}}}}");
        assert_eq!(g.metin, "2026-09-29");
        assert_eq!(g.yuvalar.len(), 1, "iç içe yuvalar dışarı sızmamalı");
    }

    #[test]
    fn kacirilmis_acilis_kapali_literaldir() {
        let g = genislet("\\{{imleç\\}}");
        assert_eq!(g.metin, "{{imleç}}");
        assert!(g.yuvalar.is_empty());
    }

    #[test]
    fn kapanmamis_acilis_hata_verir() {
        let hata = test_genisletici()
            .genislet("satir1\n{{1")
            .expect_err("kapanmamış olmalı");
        match hata {
            Hata::KapanmamisBelirtec { satir, .. } => assert_eq!(satir, 2),
            diger => panic!("beklenmeyen hata: {diger}"),
        }
    }

    #[test]
    fn asiri_kapanis_hata_verir() {
        let hata = test_genisletici()
            .genislet("metin }}")
            .expect_err("aşırı kapanış olmalı");
        assert!(matches!(hata, Hata::AsiriKapanis { .. }));
    }

    #[test]
    fn gecersiz_belirtec_reddedilir() {
        for kotu in ["{{}}", "{{0}}", "{{1|a|b|c}}", "{{1|2}}", "{{1-a}}"] {
            assert!(
                test_genisletici().genislet(kotu).is_err(),
                "{kotu} reddedilmeli"
            );
        }
    }

    #[test]
    fn odak_sirasi_konuma_gore_siralanir() {
        let g = genislet("a {{2}} b {{1}} c");
        let odak: Vec<&str> = g.odak_sirasi().iter().map(|y| y.etiket.as_str()).collect();
        assert_eq!(
            odak,
            vec!["{2}", "{1}"],
            "konum sırası korunur, indeks değil"
        );
    }

    #[test]
    fn yuvalarin_konumlari_hesaplanir() {
        let g = genislet("bir\niki {{1}}");
        assert_eq!(g.yuvalar[0].satir, 2);
        assert_eq!(g.yuvalar[0].sutun, 5);
    }

    #[test]
    fn gecersiz_deger_atamasi_ayristirilir() {
        let mut g = test_genisletici();
        assert!(g.deger_ekle("anahtarsiz").is_err());
        assert!(g.deger_ekle("=deger").is_err());
        assert!(g.deger_ekle("ad=deger").is_ok());
    }

    #[test]
    fn ad_kurallari() {
        assert!(gecerli_ad("limit"));
        assert!(gecerli_ad("_gizli1"));
        assert!(!gecerli_ad("1limit"));
        assert!(!gecerli_ad(""));
        assert!(!gecerli_ad("a-b"));
    }

    #[test]
    fn belirtec_gosterimi_yuvayi_yeniden_uretir() {
        let b = Belirtec {
            tur: BelirtecTuru::Konumsal,
            indeks: Some(1),
            ad: Some("limit".into()),
            varsayilan: Some("100".into()),
            satir: 1,
            sutun: 1,
        };
        assert_eq!(b.to_string(), "{{1|limit|100}}");
    }
}
