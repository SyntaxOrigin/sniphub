//! Ölçülebilir kazanç istatistiği: "elle kaç tuş vuruşu sürerdi".
//!
//! Kapsam: gerçek bir fiziksel tuş sayacı üzerinden kazanç hesabı. Bu, fikir
//! raporunun "ölçülebilir hız" vaadini karşılayan tek modüldür.
//!
//! # Tuş sayacı modeli
//!
//! US-QWERTY varsayımıyla her karakter 1 veya 2 fiziksel tuş vuruşudur:
//!
//! - küçük harf, rakam, boşluk ve kaydırmasız noktalama → **1**
//! - büyük harf ve `Shift` ile üretilen simgeler → **2** (`Shift` + tuş)
//! - satır sonu `\n` → **1** (`Enter`), sekme `\t` → **1** (`Tab`)
//! - ASCII dışı karakterler (Türkçe harfler dahil) → **1**
//!
//! Bilinçli sadeleştirmeler `README.md` → *Bilinen Sınırlamalar* bölümünde
//! listelenir: klavye düzeni, ölü tuşlar, IME ve otomatik girintileme
//! modellenmez.
//!
//! # Hız sabiti
//!
//! Dakikadaki kelime sayısı (WPM) kullanıcı tarafından verilir; varsayılan
//! 40 WPM'dir (WAC tipik yazma hızı). Bir kelime beş karakter kabul edildiği
//! için karakter/sn = `wpm * 5 / 60` olur.

use crate::model::Parca;

/// Varsayılan yazma hızı (WAC tipik ofis hızı).
pub const VARSAYILAN_WPM: f64 = 40.0;

/// Bir parçanın genişletilerek kazandırdığı tuş sayısı ve süre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kazanc {
    /// Gövdeyi elle yazmak için gereken tuş vuruşu.
    pub govde_tus: u64,
    /// Tetiği (ve varsa ayraç boşluğunu) yazmak için gereken tuş vuruşu.
    pub tetik_tus: u64,
    /// `govde_tus - tetik_tus` (negatif olabilir).
    pub kazanc_tus: i64,
    /// Kazancın karşılığı gelen saniye.
    pub kazanc_saniye: f64,
}

impl Kazanc {
    /// Kazanç gerçekten var mı (işe yarıyor mu).
    pub fn kazanci_var(&self) -> bool {
        self.kazanc_tus > 0
    }
}

/// Kazanç hesabının ayarları.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ayarlar {
    /// Dakikadaki kelime sayısı.
    pub wpm: f64,
    /// Tetikten sonra ayraç olarak yazılan boşluğu say (klasik genişletici
    /// geleneği: `;sorgu` + `Space`).
    pub tetik_ayraci: bool,
}

impl Default for Ayarlar {
    fn default() -> Self {
        Ayarlar {
            wpm: VARSAYILAN_WPM,
            tetik_ayraci: true,
        }
    }
}

impl Ayarlar {
    /// Karakter başına saniye.
    pub fn saniye_per_karakter(&self) -> f64 {
        if self.wpm <= 0.0 {
            return f64::INFINITY;
        }
        60.0 / (self.wpm * 5.0)
    }
}

/// Metin için fiziksel tuş vuruşu sayar.
pub fn tus_sayaci(metin: &str) -> u64 {
    let mut toplam = 0u64;
    for ch in metin.chars() {
        toplam += match ch {
            '\n' | '\t' => 1,
            c if c.is_ascii_uppercase() => 2,
            c if SHIFT_SIMGELER.contains(&c) => 2,
            _ => 1,
        };
    }
    toplam
}

/// US-QWERTY'de `Shift` ile üretilen ASCII simgeleri.
///
/// `(` ve `)` **kasten yoktur**: bunlar kaydırmasız `9` ve `0` tuşlarıdır.
/// Kaydırılmış karşılıkları `(` = `Shift+9`, `)` = `Shift+0` olduğu için tek
/// tuş vuruşudurlar.
const SHIFT_SIMGELER: [char; 19] = [
    '~', '!', '@', '#', '$', '%', '^', '&', '*', '_', '+', '{', '}', '|', ':', '"', '<', '>', '?',
];

/// Metnin ortak girinti payını çıkarır; ilk satırın girintisi atılır.
///
/// Depoda gövde girintili saklanır (fonksiyon gövdesi olabilir), kullanıcı ise
/// o girintiyi zaten yazmıştır. Kazancın abartılmaması için arındırma şarttır.
pub fn dedent(metin: &str) -> String {
    let satirlar: Vec<&str> = metin.split('\n').collect();
    if satirlar.len() < 2 {
        return metin.to_string();
    }
    let ortak = satirlar
        .iter()
        .skip(1)
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.len() - s.trim_start().len())
        .min()
        .unwrap_or(0);
    if ortak == 0 {
        return metin.to_string();
    }
    let mut cikti = String::with_capacity(metin.len());
    for (i, satir) in satirlar.iter().enumerate() {
        if i > 0 {
            cikti.push('\n');
        }
        if i == 0 {
            cikti.push_str(satir);
        } else {
            cikti.push_str(&satir[ortak.min(satir.len())..]);
        }
    }
    cikti
}

/// Bir parça için kazancı hesaplar.
pub fn hesapla(parca: &Parca, ayarlar: Ayarlar) -> Kazanc {
    let govde = parca.dedentli_govde();
    let govde_tus = tus_sayaci(&govde);
    let mut tetik_tus = tus_sayaci(&parca.tetik);
    if ayarlar.tetik_ayraci {
        tetik_tus = tetik_tus.saturating_add(1);
    }
    let kazanc_tus = i64::try_from(govde_tus)
        .unwrap_or(i64::MAX)
        .saturating_sub(i64::try_from(tetik_tus).unwrap_or(i64::MAX));
    Kazanc {
        govde_tus,
        tetik_tus,
        kazanc_tus,
        kazanc_saniye: kazanc_tus as f64 * ayarlar.saniye_per_karakter(),
    }
}

/// Depodaki toplam kazancı toplar (kullanım sayacı ağırlıklı).
pub fn toplam_kazanc(parcalar: &[Parca], ayarlar: Ayarlar) -> (u64, u64, f64) {
    let mut toplam_tus = 0u64;
    let mut toplam_kullanim = 0u64;
    let mut toplam_saniye = 0.0f64;
    for parca in parcalar {
        let k = hesapla(parca, ayarlar);
        toplam_kullanim = toplam_kullanim.saturating_add(parca.kullanim_sayaci);
        toplam_tus = toplam_tus
            .saturating_add((k.kazanc_tus.max(0) as u64).saturating_mul(parca.kullanim_sayaci));
        toplam_saniye += k.kazanc_saniye * parca.kullanim_sayaci as f64;
    }
    (toplam_tus, toplam_kullanim, toplam_saniye)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dil::Dil;
    use crate::model::Parca;
    use crate::zaman::An;

    fn parca(tetik: &str, govde: &str) -> Parca {
        Parca::yeni(
            "p",
            tetik,
            "",
            Dil::Rust,
            vec![],
            govde,
            &An::epoch_saniye(0),
        )
    }

    #[test]
    fn kucuk_harf_birer_tustur() {
        assert_eq!(tus_sayaci("abc"), 3);
        assert_eq!(tus_sayaci(" "), 1);
    }

    #[test]
    fn buyuk_harf_iki_tustur() {
        assert_eq!(tus_sayaci("A"), 2);
        assert_eq!(tus_sayaci("Ab"), 3);
    }

    #[test]
    fn kaydirilmis_simgeler_iki_tustur() {
        assert_eq!(tus_sayaci("!"), 2);
        assert_eq!(tus_sayaci("[]"), 2);
        assert_eq!(
            tus_sayaci("()"),
            2,
            "parantezler kaydırmasız 9 ve 0 tuşudur"
        );
    }

    #[test]
    fn satir_sonu_ve_sekme_birer_tustur() {
        assert_eq!(tus_sayaci("a\nb"), 3);
        assert_eq!(tus_sayaci("a\tb"), 3);
    }

    #[test]
    fn turkce_harfler_birer_tustur() {
        assert_eq!(tus_sayaci("şğüöç"), 5);
    }

    #[test]
    fn dedent_ortak_girintiyi_kaldirir() {
        assert_eq!(dedent("a\n    b\n    c"), "a\nb\nc");
        assert_eq!(dedent("a\nb"), "a\nb");
        assert_eq!(dedent("tek satir"), "tek satir");
    }

    #[test]
    fn dedent_bos_satirlari_atlamaz() {
        assert_eq!(dedent("a\n    b\n\n    c"), "a\nb\n\nc");
    }

    #[test]
    fn kazanc_elle_yazma_farki_hesaplar() {
        let p = parca(";s", "let x = 1;");
        let k = hesapla(&p, Ayarlar::default());
        // "let x = 1;" = 10 tuş; tetik ";s" = 2 + 1 ayraç = 3
        assert_eq!(k.govde_tus, 10);
        assert_eq!(k.tetik_tus, 3);
        assert_eq!(k.kazanc_tus, 7);
        assert!(k.kazanci_var());
    }

    #[test]
    fn kazanc_saniyesi_wpm_ile_olceklenir() {
        let p = parca(";s", "let x = 1;");
        let hizli = hesapla(
            &p,
            Ayarlar {
                wpm: 80.0,
                ..Ayarlar::default()
            },
        );
        let yavas = hesapla(
            &p,
            Ayarlar {
                wpm: 20.0,
                ..Ayarlar::default()
            },
        );
        assert!(hizli.kazanc_saniye < yavas.kazanc_saniye);
        assert!((hizli.kazanc_saniye - 7.0 * 60.0 / 400.0).abs() < 1e-9);
    }

    #[test]
    fn tetik_ayraci_kapatilabilir() {
        let p = parca(";s", "let x = 1;");
        let k = hesapla(
            &p,
            Ayarlar {
                tetik_ayraci: false,
                ..Ayarlar::default()
            },
        );
        assert_eq!(k.tetik_tus, 2);
    }

    #[test]
    fn kisa_parca_kazanc_saglamaz() {
        let p = parca(";uzunkisa", "x");
        let k = hesapla(&p, Ayarlar::default());
        assert!(!k.kazanci_var());
        assert!(k.kazanc_tus < 0);
    }

    #[test]
    fn toplam_kazanc_kullanim_sayacini_agir_tutar() {
        let mut a = parca(";a", "let x = 1;");
        let b = parca(";b", "let y = 2;");
        a.kullanim_sayaci = 3;
        let (tus, kullanim, saniye) = toplam_kazanc(&[a, b], Ayarlar::default());
        assert_eq!(kullanim, 3);
        // 3 kullanım × 7 tuş = 21; ikinci parça 0 kullanım
        assert_eq!(tus, 21);
        assert!(saniye > 0.0);
    }

    #[test]
    fn sifir_wpm_sonsuz_sure_verir() {
        let a = Ayarlar {
            wpm: 0.0,
            ..Ayarlar::default()
        };
        assert!(a.saniye_per_karakter().is_infinite());
    }
}
