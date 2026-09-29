//! Depo veri modeli: `Parca`, `Depo` ve doğrulama durumu.
//!
//! Kapsam: yalnızca şema ve şema üstündeki saf iş kuralları (ad benzersizliği,
//! kimlik üretimi, sıralama). Dosya giriş/çıkışı `depo.rs` modülündedir.
//!
//! **Güvenlik notu (rapor riski, bilinçli sapma):** Parça gövdeleri düz
//! metin saklanır. Şifreleme, Argon2id + XChaCha20 olarak ertelenmiştir
//! (WORKER_CONTRACT.md § 3.2-C: bu kripto crate'leri yalnız 16/17/30'a
//! serbest). Kullanıcı bu aracı parola/anahtar saklamak için kullanmamalıdır.

use crate::dil::Dil;
use crate::hata::{Hata, Sonuc};
use crate::zaman::An;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Depo dosyasının okunabilen/yazılabilen en güncel biçim sürümü.
pub const DEPO_SURUMU: u32 = 1;

/// Bir parçanın yapısal denetim sonucu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dogrulama {
    /// Henüz denetlenmemiş.
    #[default]
    Denenmemis,
    /// Yapısal denge kontrolünden geçti.
    Gecerli,
    /// Yapısal denge kontrolünde hata buldu.
    Hatali,
}

impl Dogrulama {
    /// Arama sıralamasında kullanılan puan katkısı.
    ///
    /// Fikir raporu § 03 onay kriteri: hatalı parça arama sonuçlarında
    /// *son sırada* çıkar.
    pub const fn arama_puani(self) -> i32 {
        match self {
            Dogrulama::Gecerli => 5,
            Dogrulama::Denenmemis => 0,
            Dogrulama::Hatali => -30,
        }
    }

    /// Depo dosyasında ve dışa aktarımda kullanılan kanonik yazım.
    pub const fn etiket(self) -> &'static str {
        match self {
            Dogrulama::Denenmemis => "denenmemis",
            Dogrulama::Gecerli => "gecerli",
            Dogrulama::Hatali => "hatali",
        }
    }
}

impl std::fmt::Display for Dogrulama {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.etiket())
    }
}

/// Depoda saklanan tek bir programlama parçası.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parca {
    /// Gövdeden türetilen kararlı kimlik (slug + sağlama).
    pub id: String,
    /// Kullanıcıya gösterilen benzersiz ad.
    pub ad: String,
    /// `expand` komutunun tetiği.
    pub tetik: String,
    /// Tek satırlık açıklama.
    pub aciklama: String,
    /// Yazıldığı dil.
    pub dil: Dil,
    /// Serbest etiket listesi.
    pub etiketler: Vec<String>,
    /// Genişletilecek kod gövdesi.
    pub govde: String,
    /// Oluşturma zaman damgası (ISO-8601 UTC).
    pub olusturma: String,
    /// Son güncelleme zaman damgası (ISO-8601 UTC).
    pub guncelleme: String,
    /// Kaç kez genişletildiği.
    pub kullanim_sayaci: u64,
    /// Son yapısal denetim sonucu.
    pub dogrulama: Dogrulama,
}

impl Parca {
    /// Ad ve gövdeden kararlı (rastgelelik olmadan) bir kimlik üretir.
    ///
    /// FNV-1a 64-bit sağlamasının alt 32 biti kullanılır: kimlik insan tarafından
    /// okunup karşılaştırılabilir olmalı, tam 64 hane gereksiz gürültü yaratır.
    /// Bu, `add` komutunun idempotent olmasını sağlar.
    pub fn kimlik_uret(ad: &str, govde: &str) -> String {
        let ozet = fnv1a64(&format!("{ad}\u{0}{govde}")) & 0xffff_ffff;
        format!("{}-{:08x}", slug(ad), ozet)
    }

    /// Verilen zaman görüntüsüyle yeni bir parça oluşturur.
    #[allow(clippy::too_many_arguments)]
    pub fn yeni(
        ad: &str,
        tetik: &str,
        aciklama: &str,
        dil: Dil,
        etiketler: Vec<String>,
        govde: &str,
        an: &An,
    ) -> Self {
        Parca {
            id: Parca::kimlik_uret(ad, govde),
            ad: ad.to_string(),
            tetik: tetik.to_string(),
            aciklama: aciklama.to_string(),
            dil,
            etiketler,
            govde: govde.to_string(),
            olusturma: an.zaman.clone(),
            guncelleme: an.zaman.clone(),
            kullanim_sayaci: 0,
            dogrulama: Dogrulama::Denenmemis,
        }
    }

    /// Kullanım sayacını bir artırır ve güncelleme damgasını yeniler.
    pub fn kullanim_artir(&mut self, an: &An) {
        self.kullanim_sayaci = self.kullanim_sayaci.saturating_add(1);
        self.guncelleme = an.zaman.clone();
    }

    /// Gövdeyi ortak girinti payından arındırır (ilk satır hariç).
    ///
    /// Kazanç hesabının dürüst olması için gereklidir: bir fonksiyon gövdesi
    /// depoda girintili saklanır, kullanıcı ise girintiyi zaten yazmış olur.
    pub fn dedentli_govde(&self) -> String {
        crate::kazanc::dedent(&self.govde)
    }

    /// Verilen anahtarla eşleşiyor mu (ad büyük/küçük harf duyarsız, tetik birebir).
    pub fn bul_anahtarla(&self, anahtar: &str) -> bool {
        self.ad.to_lowercase() == anahtar.to_lowercase() || self.tetik == anahtar
    }
}

/// Parça koleksiyonu — depo dosyasının kök şeması.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Depo {
    /// Biçim sürümü.
    pub surum: u32,
    /// Parça listesi.
    pub parcalar: Vec<Parca>,
}

impl Default for Depo {
    fn default() -> Self {
        Depo {
            surum: DEPO_SURUMU,
            parcalar: Vec::new(),
        }
    }
}

impl Depo {
    /// Boş bir depo üretir.
    pub fn bos() -> Self {
        Depo::default()
    }

    /// Adı büyük/küçük harf duyarsız olarak arar.
    pub fn adla(&self, ad: &str) -> Option<&Parca> {
        let hedef = ad.to_lowercase();
        self.parcalar.iter().find(|p| p.ad.to_lowercase() == hedef)
    }

    /// Tetiği büyük/küçük harf duyarsız olarak arar.
    pub fn tetikle(&self, tetik: &str) -> Option<&Parca> {
        self.parcalar.iter().find(|p| p.tetik == tetik)
    }

    /// Ad ya da tetik ile arar; önce ada bakar.
    pub fn bul(&self, anahtar: &str) -> Option<&Parca> {
        self.adla(anahtar).or_else(|| self.tetikle(anahtar))
    }

    /// Aynı adda ikinci bir parça eklemeyi reddeder.
    pub fn ekle(&mut self, parca: Parca) -> Sonuc<()> {
        if self.parcalar.iter().any(|p| p.ad == parca.ad) {
            return Err(Hata::YinelenenAd { ad: parca.ad });
        }
        self.parcalar.push(parca);
        self.parcalar.sort_by(|a, b| {
            a.ad.to_lowercase()
                .cmp(&b.ad.to_lowercase())
                .then(a.id.cmp(&b.id))
        });
        Ok(())
    }

    /// Adı veya tetiği eşleşen parçayı bulur; yoksa hata döner.
    pub fn bul_zorunlu(&self, anahtar: &str) -> Sonuc<&Parca> {
        self.bul(anahtar).ok_or_else(|| Hata::ParcaYok {
            anahtar: anahtar.to_string(),
        })
    }

    /// Şema bütünlüğünü denetler: sürüm, benzersiz ad, benzersiz kimlik.
    pub fn dogrula(&self) -> Sonuc<()> {
        if self.surum != DEPO_SURUMU {
            return Err(Hata::DesteklenmeyenSurum {
                bulunan: self.surum,
                desteklenen: DEPO_SURUMU,
            });
        }
        let mut adlar = BTreeMap::new();
        let mut kimlikler = BTreeMap::new();
        for (i, parca) in self.parcalar.iter().enumerate() {
            if parca.ad.trim().is_empty() {
                return Err(Hata::BozukDepo {
                    alan: format!("parcalar[{i}].ad"),
                    neden: "ad boş olamaz".into(),
                });
            }
            if parca.tetik.trim().is_empty() {
                return Err(Hata::BozukDepo {
                    alan: format!("parcalar[{i}].tetik"),
                    neden: "tetik boş olamaz".into(),
                });
            }
            if parca.govde.is_empty() {
                return Err(Hata::BozukDepo {
                    alan: format!("parcalar[{i}].govde"),
                    neden: "gövde boş olamaz".into(),
                });
            }
            if adlar.insert(parca.ad.to_lowercase(), i).is_some() {
                return Err(Hata::BozukDepo {
                    alan: format!("parcalar[{i}].ad"),
                    neden: format!("ad yineleniyor: {}", parca.ad),
                });
            }
            if kimlikler.insert(parca.id.as_str(), i).is_some() {
                return Err(Hata::BozukDepo {
                    alan: format!("parcalar[{i}].id"),
                    neden: format!("kimlik yineleniyor: {}", parca.id),
                });
            }
        }
        Ok(())
    }
}

/// Ada dönüşen, kısa, ASCII ve URL/grafik uyumlu slug üretir.
///
/// Türkçe harfler ASCII karşılıklarına çevrilir; aksi hâlde "başlık" gibi bir
/// ad `bas-lik` gibi kırılır ve kimlikler okunmaz hâle gelir.
pub fn slug(ad: &str) -> String {
    let mut cikti = String::new();
    let mut son_tire = true;
    for ch in ad.chars().flat_map(ascii_karsilik) {
        if ch.is_ascii_alphanumeric() {
            cikti.push(ch.to_ascii_lowercase());
            son_tire = false;
        } else if !son_tire {
            cikti.push('-');
            son_tire = true;
        }
    }
    while cikti.ends_with('-') {
        cikti.pop();
    }
    if cikti.is_empty() {
        cikti.push_str("parca");
    }
    if cikti.len() > 40 {
        cikti.truncate(40);
        while cikti.ends_with('-') {
            cikti.pop();
        }
    }
    cikti
}

/// Bir karakteri ASCII karşılıklarına açar; ASCII olmayan harfler elenir.
fn ascii_karsilik(ch: char) -> Vec<char> {
    match ch.to_lowercase().next().unwrap_or(ch) {
        'ç' => "c".chars().collect(),
        'ğ' => "g".chars().collect(),
        'ı' => "i".chars().collect(),
        'ö' => "o".chars().collect(),
        'ş' => "s".chars().collect(),
        'ü' => "u".chars().collect(),
        'â' | 'î' | 'û' => Vec::new(),
        _ => vec![ch],
    }
}

/// FNV-1a 64-bit sağlaması (bağımlılıksız, kararlı).
pub fn fnv1a64(veri: &str) -> u64 {
    const KIRMA: u64 = 0xcbf2_9ce4_8422_2325;
    let mut h = KIRMA;
    for bayt in veri.as_bytes() {
        h ^= u64::from(*bayt);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ornek() -> Parca {
        Parca::yeni(
            "Sorgu başlığı",
            ";sorgu",
            "SQL başlığı",
            Dil::Sql,
            vec!["sql".into(), "sorgu".into()],
            "SELECT 1;",
            &An::epoch_saniye(0),
        )
    }

    #[test]
    fn yeni_parca_alanlari_dogru() {
        let p = ornek();
        assert_eq!(p.ad, "Sorgu başlığı");
        assert_eq!(p.dil, Dil::Sql);
        assert_eq!(p.kullanim_sayaci, 0);
        assert_eq!(p.dogrulama, Dogrulama::Denenmemis);
        assert_eq!(p.olusturma, "1970-01-01T00:00:00Z");
        assert!(
            p.id.starts_with("sorgu-basligi-") && p.id.len() == "sorgu-basligi-".len() + 8,
            "kimlik biçimi bozuk: {}",
            p.id
        );
    }

    #[test]
    fn ayni_ad_iki_parca_reddedilir() {
        let mut depo = Depo::bos();
        depo.ekle(ornek()).expect("ilk ekleme");
        let hata = depo.ekle(ornek()).expect_err("ikinci ekleme reddedilmeli");
        assert!(matches!(hata, Hata::YinelenenAd { .. }));
    }

    #[test]
    fn kimlik_kararli_ve_govdeden_bağımlı() {
        assert_eq!(
            Parca::kimlik_uret("a", "b"),
            Parca::kimlik_uret("a", "b"),
            "aynı girdi aynı kimliği vermeli"
        );
        assert_ne!(Parca::kimlik_uret("a", "b"), Parca::kimlik_uret("a", "c"));
    }

    #[test]
    fn slug_kural_larini_temizler() {
        assert_eq!(slug("Sorgu Başlığı"), "sorgu-basligi");
        assert_eq!(slug("   "), "parca");
        assert!(
            slug(&"çok-uzun-ad-".repeat(20)).len() <= 40,
            "slug 40 karakteri aşmamalı"
        );
    }

    #[test]
    fn kullanim_sayaci_ve_damga_guncellenir() {
        let mut p = ornek();
        p.kullanim_artir(&An::epoch_saniye(86_400));
        assert_eq!(p.kullanim_sayaci, 1);
        assert_eq!(p.guncelleme, "1970-01-02T00:00:00Z");
    }

    #[test]
    fn arama_hem_ad_hem_tetikle_calisir() {
        let mut depo = Depo::bos();
        depo.ekle(ornek()).expect("ekleme");
        assert!(depo.bul("sorgu başlığı").is_some());
        assert!(depo.bul("Sorgu Başlığı").is_some());
        assert!(depo.bul(";sorgu").is_some());
        assert!(depo.bul(";yok").is_none());
        assert!(depo.bul_zorunlu(";yok").is_err());
    }

    #[test]
    fn turkce_noktasiz_i_buyuk_kucuk_harf_sinirlamasi_belgelidir() {
        // Rust'in `to_lowercase` eşlemesi ASCII 'I' harfini 'i' yapar; Türkçe'de
        // 'ı' beklenir. Bu araç Unicode varsayılan eşlemesini kullanır ve
        // noktasız/dotlu i ayrımı yapmaz — bilinçli ve belgelenmiş bir sınırdır.
        let mut depo = Depo::bos();
        depo.ekle(ornek()).expect("ekleme");
        assert!(
            depo.bul("BAŞLIĞI").is_none(),
            "noktasız i yanlış eşleşmemeli"
        );
        assert!(
            depo.bul("Sorgu Başlığı").is_some(),
            "ASCII büyük/küçük harf çalışır"
        );
    }

    #[test]
    fn bozuk_depo_sema_denetimi_chaidirir() {
        let mut depo = Depo::bos();
        depo.ekle(ornek()).expect("ekleme");
        let mut kopya = depo.clone();
        kopya.surum = 99;
        assert!(matches!(
            kopya.dogrula(),
            Err(Hata::DesteklenmeyenSurum { bulunan: 99, .. })
        ));

        let mut kopya = depo.clone();
        kopya.parcalar.push(ornek());
        assert!(matches!(kopya.dogrula(), Err(Hata::BozukDepo { .. })));

        let mut kopya = depo;
        kopya.parcalar[0].govde.clear();
        assert!(matches!(kopya.dogrula(), Err(Hata::BozukDepo { .. })));
    }

    #[test]
    fn dogrulama_puani_hataliyi_sona_atar() {
        assert!(Dogrulama::Hatali.arama_puani() < Dogrulama::Denenmemis.arama_puani());
        assert!(Dogrulama::Gecerli.arama_puani() > Dogrulama::Denenmemis.arama_puani());
    }
}
