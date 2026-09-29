//! Kullanım ve kazanç istatistiği.
//!
//! Kapsam: toplamlar, dil dağılımı ve "en çok kullanılan" listesi. Hesabın
//! kendisi `kazanc.rs` modülündedir; burada yalnızca raporlama vardır.

use crate::dil::Dil;
use crate::kazanc::{self, Ayarlar};
use crate::model::Parca;
use std::fmt;

/// Bir parçanın kullanım satırı.
#[derive(Debug, Clone, PartialEq)]
pub struct KullanimSatiri {
    /// Parça adı.
    pub ad: String,
    /// Kaç kez genişletildi.
    pub sayi: u64,
    /// Tek kullanımda kazanan tuş vuruşu.
    pub kazanc_tus: i64,
    /// Kümülatif kazanan tuş vuruşu.
    pub toplam_tus: u64,
}

impl fmt::Display for KullanimSatiri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<28} {:>5} kez  {:>7} tuş/kayıt  {:>9} tuş toplam",
            self.ad, self.sayi, self.kazanc_tus, self.toplam_tus
        )
    }
}

/// Depo istatistiğinin tamamı.
#[derive(Debug, Clone, PartialEq)]
pub struct Istatistik {
    /// Toplam parça sayısı.
    pub parca_sayisi: usize,
    /// Dil bazında dağılım, ada göre sıralı.
    pub dagilim: Vec<(Dil, usize)>,
    /// Toplam genişletme sayısı.
    pub toplam_kullanim: u64,
    /// Toplam kazanan tuş vuruşu.
    pub toplam_kazanc_tus: u64,
    /// Toplam kazanan süre (saniye).
    pub toplam_kazanc_saniye: f64,
    /// Kazancı olmayan parça sayısı (kısa tetik, uzun gövde yok).
    pub kazanci_olmayan: usize,
    /// En çok kullanılan parçalar.
    pub en_cok: Vec<KullanimSatiri>,
}

/// İstatistiği hesaplar.
///
/// `en_cok` listesi puana, ardından ada göre kararlı sıralanır; aynı depo her
/// zaman aynı tabloyu üretir.
pub fn hesapla(parcalar: &[Parca], ayarlar: Ayarlar, en_cok_adet: usize) -> Istatistik {
    let (toplam_kazanc_tus, toplam_kullanim, toplam_kazanc_saniye) =
        kazanc::toplam_kazanc(parcalar, ayarlar);

    let mut sayaclar: Vec<(Dil, usize)> = Vec::new();
    for parca in parcalar {
        match sayaclar.iter_mut().find(|(d, _)| *d == parca.dil) {
            Some((_, adet)) => *adet += 1,
            None => sayaclar.push((parca.dil, 1)),
        }
    }
    sayaclar.sort_by_key(|(d, _)| d.etiket());

    let kazanci_olmayan = parcalar
        .iter()
        .filter(|p| !kazanc::hesapla(p, ayarlar).kazanci_var())
        .count();

    let mut satirlar: Vec<KullanimSatiri> = parcalar
        .iter()
        .map(|p| {
            let k = kazanc::hesapla(p, ayarlar);
            KullanimSatiri {
                ad: p.ad.clone(),
                sayi: p.kullanim_sayaci,
                kazanc_tus: k.kazanc_tus,
                toplam_tus: (k.kazanc_tus.max(0) as u64).saturating_mul(p.kullanim_sayaci),
            }
        })
        .collect();
    satirlar.sort_by(|a, b| b.sayi.cmp(&a.sayi).then_with(|| a.ad.cmp(&b.ad)));
    satirlar.retain(|s| s.sayi > 0);
    satirlar.truncate(en_cok_adet);

    Istatistik {
        parca_sayisi: parcalar.len(),
        dagilim: sayaclar,
        toplam_kullanim,
        toplam_kazanc_tus,
        toplam_kazanc_saniye,
        kazanci_olmayan,
        en_cok: satirlar,
    }
}

impl Istatistik {
    /// Toplam kazancın dakika cinsinden gösterimi.
    pub fn toplam_dakika(&self) -> f64 {
        self.toplam_kazanc_saniye / 60.0
    }

    /// Kazancı olmayan parçaların oranı (0.0–1.0).
    pub fn ise_yaramaz_orani(&self) -> f64 {
        if self.parca_sayisi == 0 {
            return 0.0;
        }
        f64::from(u16::try_from(self.kazanci_olmayan).unwrap_or(u16::MAX))
            / self.parca_sayisi as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Parca;
    use crate::zaman::An;

    fn parca(ad: &str, tetik: &str, dil: Dil, govde: &str, sayi: u64) -> Parca {
        let mut p = Parca::yeni(ad, tetik, "", dil, vec![], govde, &An::epoch_saniye(0));
        p.kullanim_sayaci = sayi;
        p
    }

    #[test]
    fn dil_dagilimi_sayilir_ve_siralanir() {
        let parcalar = vec![
            parca("A", ";a", Dil::Rust, "fn a() {}", 3),
            parca("B", ";b", Dil::Python, "def b():\n    pass\n", 1),
            parca("C", ";c", Dil::Rust, "fn c() {}", 0),
        ];
        let i = hesapla(&parcalar, Ayarlar::default(), 20);
        assert_eq!(i.parca_sayisi, 3);
        assert_eq!(i.dagilim, vec![(Dil::Python, 1), (Dil::Rust, 2)]);
    }

    #[test]
    fn en_cok_siralamasi_kararlidir() {
        let parcalar = vec![
            parca("A", ";a", Dil::Rust, "fn a() {}", 3),
            parca("B", ";b", Dil::Rust, "fn b() {}", 3),
            parca("C", ";c", Dil::Rust, "fn c() {}", 9),
        ];
        let i = hesapla(&parcalar, Ayarlar::default(), 2);
        assert_eq!(i.en_cok[0].ad, "C");
        assert_eq!(i.en_cok[1].ad, "A", "eşitlikte ada göre sıralanır");
    }

    #[test]
    fn kullanilmayan_parcalar_listelenmez() {
        let parcalar = vec![parca("A", ";a", Dil::Rust, "fn a() {}", 0)];
        let i = hesapla(&parcalar, Ayarlar::default(), 20);
        assert!(i.en_cok.is_empty());
        assert_eq!(i.toplam_kullanim, 0);
    }

    #[test]
    fn kazanci_olmayan_parca_sayilir() {
        let parcalar = vec![
            parca("A", ";a", Dil::Rust, "let x = 1;", 1),
            parca("B", ";cokuzunkisa", Dil::Rust, "x", 1),
        ];
        let i = hesapla(&parcalar, Ayarlar::default(), 20);
        assert_eq!(i.kazanci_olmayan, 1);
        assert!((i.ise_yaramaz_orani() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn bos_depo_istatistigi_sifirdir() {
        let i = hesapla(&[], Ayarlar::default(), 20);
        assert_eq!(i.parca_sayisi, 0);
        assert_eq!(i.toplam_kazanc_tus, 0);
        assert!(i.toplam_dakika().abs() < 1e-9);
        assert!(i.ise_yaramaz_orani().abs() < 1e-9);
    }

    #[test]
    fn toplam_kazanc_kullanimla_carpar() {
        let parcalar = vec![parca("A", ";a", Dil::Rust, "let x = 1;", 2)];
        let i = hesapla(&parcalar, Ayarlar::default(), 20);
        assert_eq!(i.en_cok[0].kazanc_tus, 7);
        assert_eq!(i.en_cok[0].toplam_tus, 14);
        assert_eq!(i.toplam_kazanc_tus, 14);
    }

    #[test]
    fn satir_gosterimi_okunabilir() {
        let s = KullanimSatiri {
            ad: "Test".into(),
            sayi: 2,
            kazanc_tus: 10,
            toplam_tus: 20,
        };
        let metin = s.to_string();
        assert!(metin.contains("Test"));
        assert!(metin.contains("20 tuş toplam"));
    }
}
