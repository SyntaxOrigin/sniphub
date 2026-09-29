//! Çoklu dil arama, filtreleme ve alaka sıralaması.
//!
//! Kapsam: sorgu kurulumu, filtreleme ve **kararlı** (deterministik) sıralama.
//! Sıralama eşitlik bozulduğunda ada ve kimliğe göre ikinci kırılım uygular;
//! aynı depo her zaman aynı sırayı verir.
//!
//! Fikir raporu § 03 onay kriteri: hatalı parça arama sonuçlarında son sırada
//! çıkar. Bu kural [`crate::model::Dogrulama::arama_puani`] ile puan tabanlı
//! olarak uygulanır.

use crate::dil::Dil;
use crate::model::Parca;

/// Bir alanın sıralama ağırlığı.
const AD_TAM: i32 = 100;
const TETIK_TAM: i32 = 90;
const AD_BASLANGIC: i32 = 60;
const TETIK_BASLANGIC: i32 = 50;
const ETIKET_TAM: i32 = 45;
const AD_ICINDE: i32 = 40;
const GOVDE_ICINDE: i32 = 10;
const ACIKLAMA_ICINDE: i32 = 15;
const ID_ICINDE: i32 = 20;
const KUCUK_HARF_BONUS: i32 = 2;

/// Arama ve filtreleme sorgusu.
#[derive(Debug, Clone, Default)]
pub struct Sorgu {
    /// Serbest metin; ad, tetik, etiket, açıklama ve gövdede aranır.
    pub metin: Option<String>,
    /// Yalnızca bu dildeki parçalar.
    pub dil: Option<Dil>,
    /// Gerekli etiketler.
    pub etiketler: Vec<String>,
    /// `true` ise etiketlerin **hepsi** eşleşmeli (AND), `false` ise en az biri.
    pub her_etiket: bool,
}

impl Sorgu {
    /// Serbest metinle arama sorgusu kurar.
    pub fn metin(metin: &str) -> Self {
        Sorgu {
            metin: Some(metin.to_lowercase()),
            ..Sorgu::default()
        }
    }

    /// Dile göre filtre uygular.
    pub fn dil_ile(mut self, dil: Option<Dil>) -> Self {
        self.dil = dil;
        self
    }

    /// Etiket listesini ayarlar ve eşleştirme kiplerini seçer.
    pub fn etiketler_ile(mut self, etiketler: Vec<String>, her_biri: bool) -> Self {
        self.etiketler = etiketler
            .into_iter()
            .map(|e| e.trim().to_lowercase())
            .filter(|e| !e.is_empty())
            .collect();
        self.her_etiket = her_biri;
        self
    }
}

/// Tek bir arama sonucu: parça ve puanı.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sonuc<'a> {
    /// Eşleşen parça.
    pub parca: &'a Parca,
    /// Alaka puanı.
    pub puan: i32,
    /// Puanı oluşturan gerekçeler (kullanıcıya gösterilir).
    pub gerekce: Vec<String>,
}

/// Sorguyu karşılayan parçaları, alaka sırasına göre döner.
pub fn ara<'a>(parcalar: &'a [Parca], sorgu: &Sorgu) -> Vec<Sonuc<'a>> {
    let mut sonuclar: Vec<Sonuc<'a>> = parcalar
        .iter()
        .filter(|p| filtreler(p, sorgu))
        .filter_map(|p| {
            puanla(p, sorgu).map(|puan| Sonuc {
                parca: p,
                puan: puan.0,
                gerekce: puan.1,
            })
        })
        .collect();
    sonuclar.sort_by(|a, b| {
        b.puan
            .cmp(&a.puan)
            .then_with(|| a.parca.ad.to_lowercase().cmp(&b.parca.ad.to_lowercase()))
            .then_with(|| a.parca.id.cmp(&b.parca.id))
    });
    sonuclar
}

/// Filtreleri uygular; `true` ise parça sorgunun kapsamındadır.
fn filtreler(parca: &Parca, sorgu: &Sorgu) -> bool {
    if let Some(dil) = sorgu.dil {
        if parca.dil != dil {
            return false;
        }
    }
    if sorgu.etiketler.is_empty() {
        return true;
    }
    let sahip: Vec<String> = parca.etiketler.iter().map(|e| e.to_lowercase()).collect();
    if sorgu.her_etiket {
        sorgu.etiketler.iter().all(|a| sahip.contains(a))
    } else {
        sorgu.etiketler.iter().any(|a| sahip.contains(a))
    }
}

/// Bir parçanın puanını ve gerekçesini hesaplar; eşleşme yoksa `None`.
fn puanla(parca: &Parca, sorgu: &Sorgu) -> Option<(i32, Vec<String>)> {
    let (metin, gerekce) = match &sorgu.metin {
        None => (0, Vec::new()),
        Some(aranan) if aranan.is_empty() => (0, Vec::new()),
        Some(aranan) => {
            let ad = parca.ad.to_lowercase();
            let tetik = parca.tetik.to_lowercase();
            let aciklama = parca.aciklama.to_lowercase();
            let govde = parca.govde.to_lowercase();
            let id = parca.id.to_lowercase();
            let etiketler: Vec<String> = parca.etiketler.iter().map(|e| e.to_lowercase()).collect();

            let mut puan = 0;
            let mut gerekce = Vec::new();
            if ad == *aranan {
                puan += AD_TAM;
                gerekce.push("ad tam eşleşti".into());
            } else if ad.starts_with(aranan.as_str()) {
                puan += AD_BASLANGIC;
                gerekce.push("ad ile başlıyor".into());
            } else if ad.contains(aranan.as_str()) {
                puan += AD_ICINDE;
                gerekce.push("ad içinde geçiyor".into());
            }
            if tetik == *aranan {
                puan += TETIK_TAM;
                gerekce.push("tetik tam eşleşti".into());
            } else if tetik.starts_with(aranan.as_str()) {
                puan += TETIK_BASLANGIC;
                gerekce.push("tetik ile başlıyor".into());
            }
            if etiketler.iter().any(|e| e == aranan) {
                puan += ETIKET_TAM;
                gerekce.push("etiket tam eşleşti".into());
            }
            if aciklama.contains(aranan.as_str()) {
                puan += ACIKLAMA_ICINDE;
                gerekce.push("açıklamada geçiyor".into());
            }
            if id.contains(aranan.as_str()) {
                puan += ID_ICINDE;
                gerekce.push("kimlikte geçiyor".into());
            }
            if govde.contains(aranan.as_str()) {
                puan += GOVDE_ICINDE;
                gerekce.push("gövdede geçiyor".into());
            }
            if puan == 0 {
                return None;
            }
            if puan <= GOVDE_ICINDE {
                puan += KUCUK_HARF_BONUS;
            }
            (puan, gerekce)
        }
    };
    let puan = metin + parca.dogrulama.arama_puani();
    Some((puan, gerekce))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Dogrulama, Parca};
    use crate::zaman::An;

    fn parca(ad: &str, tetik: &str, dil: Dil, etiketler: &[&str], govde: &str) -> Parca {
        let mut p = Parca::yeni(
            ad,
            tetik,
            "",
            dil,
            etiketler.iter().map(|e| (*e).to_string()).collect(),
            govde,
            &An::epoch_saniye(0),
        );
        p.dogrulama = Dogrulama::Gecerli;
        p
    }

    /// Adı `Sorgu Başlığı` olan, `sql` etiketli ve `SELECT ... sorgu` gövdeli parça.
    fn depo_ornegi() -> Vec<Parca> {
        vec![
            parca(
                "Sorgu Başlığı",
                ";sorgu",
                Dil::Sql,
                &["sql", "db"],
                "SELECT * FROM sorgu",
            ),
            parca(
                "Rust hata yakalama",
                ";hat",
                Dil::Rust,
                &["rust", "hata"],
                "fn f() {}",
            ),
            parca(
                "Sorgu tekrar",
                ";tekrar",
                Dil::Rust,
                &["rust"],
                "let x = 1;",
            ),
        ]
    }

    #[test]
    fn metin_arama_ad_uyumuna_gore_siralanir() {
        let parcalar = depo_ornegi();
        let s = ara(&parcalar, &Sorgu::metin("sorgu"));
        // "Sorgu Başlığı" (başlangıç 60) ve "Sorgu tekrar" (başlangıç 60).
        assert_eq!(s.len(), 2, "gövdede geçenler de eşleşmelidir ama yok");
        // Eşit puanlı iki sonuç ada göre sıralanır: "Sorgu Başlığı" < "Sorgu tekrar"
        assert_eq!(s[0].parca.ad, "Sorgu Başlığı");
        assert_eq!(s[1].parca.ad, "Sorgu tekrar");
    }

    #[test]
    fn govde_icerigi_dusuk_puanla_eslesir() {
        let parcalar = depo_ornegi();
        let s = ara(&parcalar, &Sorgu::metin("select"));
        assert_eq!(s.len(), 1);
        assert!(s[0].gerekce.iter().any(|g| g == "gövdede geçiyor"));
        assert_eq!(s[0].puan, GOVDE_ICINDE + KUCUK_HARF_BONUS + 5);
    }

    #[test]
    fn ad_tam_eslesmesi_en_yuksek_puani_alir() {
        let parcalar = depo_ornegi();
        let s = ara(&parcalar, &Sorgu::metin("sorgu başlığı"));
        assert_eq!(s[0].parca.ad, "Sorgu Başlığı");
        assert!(s[0].gerekce.iter().any(|g| g == "ad tam eşleşti"));
        // ad tam eşleşmesi (100) + yapısal denge puanı (5); açıklama ve etiket
        // eşleşmediği için başka katkı yoktur.
        assert_eq!(s[0].puan, AD_TAM + Dogrulama::Gecerli.arama_puani());
    }

    #[test]
    fn gerekce_tetik_ve_etiket_eslesmelerini_yazar() {
        let parcalar = vec![parca("Başlık", "sorgu;", Dil::Sql, &["sorgu"], "SELECT 1")];
        let s = ara(&parcalar, &Sorgu::metin("sorgu"));
        assert_eq!(s[0].gerekce.len(), 2, "{:?}", s[0].gerekce);
        assert!(s[0].gerekce.iter().any(|g| g == "tetik ile başlıyor"));
        assert!(s[0].gerekce.iter().any(|g| g == "etiket tam eşleşti"));
    }

    #[test]
    fn dil_filtresi_calisir() {
        let parcalar = depo_ornegi();
        let s = ara(&parcalar, &Sorgu::metin("sorgu").dil_ile(Some(Dil::Rust)));
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].parca.dil, Dil::Rust);
    }
    #[test]
    fn etiket_filtresi_ve_ve_kipleri() {
        let parcalar = depo_ornegi();
        let s = ara(
            &parcalar,
            &Sorgu::default().etiketler_ile(vec!["rust".into()], true),
        );
        assert_eq!(s.len(), 2);

        let s = ara(
            &parcalar,
            &Sorgu::default().etiketler_ile(vec!["sql".into(), "rust".into()], true),
        );
        assert!(s.is_empty(), "AND kipi: ikisi birden yok");

        let s = ara(
            &parcalar,
            &Sorgu::default().etiketler_ile(vec!["sql".into(), "rust".into()], false),
        );
        assert_eq!(s.len(), 3, "OR kipi: en az biri");
    }

    #[test]
    fn hatali_parca_sonda_cikar() {
        let mut parcalar = depo_ornegi();
        parcalar[2].dogrulama = Dogrulama::Hatali;
        let s = ara(&parcalar, &Sorgu::metin("sorgu"));
        assert_eq!(
            s.last().map(|x| x.parca.ad.clone()),
            Some("Sorgu tekrar".to_string()),
            "hatalı parça son sırada olmalı"
        );
    }
    #[test]
    fn siralama_kararlidir() {
        let parcalar = depo_ornegi();
        let a = ara(&parcalar, &Sorgu::metin("a"));
        let b = ara(&parcalar, &Sorgu::metin("a"));
        assert_eq!(
            a.iter().map(|x| &x.parca.id).collect::<Vec<_>>(),
            b.iter().map(|x| &x.parca.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn bulunmayan_metin_sonuc_dondurmez() {
        let parcalar = depo_ornegi();
        assert!(ara(&parcalar, &Sorgu::metin("bulunmayan")).is_empty());
    }

    #[test]
    fn bos_sorgu_tum_parcalari_dondurur() {
        let parcalar = depo_ornegi();
        assert_eq!(ara(&parcalar, &Sorgu::default()).len(), 3);
        assert_eq!(ara(&parcalar, &Sorgu::metin("")).len(), 3);
    }

    #[test]
    fn buyuk_kucuk_harf_duyarsizdir() {
        let parcalar = depo_ornegi();
        assert_eq!(ara(&parcalar, &Sorgu::metin("SQL")).len(), 1);
        assert_eq!(ara(&parcalar, &Sorgu::metin("hat")).len(), 1);
    }
}
