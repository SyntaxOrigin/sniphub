//! Düz metin parça deposu: okuma, yazma ve atomiklik.
//!
//! Kapsam: `parcalar.json` dosyasının şema denetimi, atomik yazımı ve
//! güvenli okuması. Arama ve sıralama `arama.rs`, kazanç hesabı `kazanc.rs`
//! modülündedir.
//!
//! **Şifreleme yoktur** (MANIFEST.md kartı, "Sapma gerekçesi"). Argon2id +
//! XChaCha20 yerine düz metin JSON kullanılır; bu, ürünün en önemli güvenlik
//! eksiğidir ve README'nin en üstünde yazılıdır. Kullanıcı bu depoya parola,
//! API anahtarı veya benzeri gizli değer koymamalıdır.

use crate::hata::{Hata, Sonuc};
use crate::model::{Depo, DEPO_SURUMU};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

/// Depo dosyasının varsayılan adı.
pub const VARSAYILAN_DEPO_ADI: &str = "parcalar.json";

/// Depo dosyasının okunabilmesi için gereken en küçük boyut.
const MIN_BOYUT: u64 = 2;

/// Depoyu verilen yoldan okur ve şema denetiminden geçirir.
///
/// Bozuk JSON, desteklenmeyen sürüm ve şema ihlali ayrı ayrı hata döner;
/// hiçbir durumda `panic` üretilmez.
pub fn yukle(yol: &Path) -> Sonuc<Depo> {
    let meta = fs::metadata(yol).map_err(|kaynak| Hata::io("depo okunamadı", yol, kaynak))?;
    if meta.len() < MIN_BOYUT {
        return Err(Hata::BozukDepo {
            alan: yol.display().to_string(),
            neden: "dosya boş veya eksik".into(),
        });
    }
    let metin =
        fs::read_to_string(yol).map_err(|kaynak| Hata::io("depo okunamadı", yol, kaynak))?;
    let depo: Depo =
        serde_json::from_str(&metin).map_err(|kaynak| Hata::json("depo çözümlenemedi", kaynak))?;
    depo.dogrula()?;
    Ok(depo)
}

/// Depoyu verilen yola **atomik** olarak yazar.
///
/// Yazma iki adımdan oluşur: önce aynı dizinde `*.tmp` adlı geçici dosya
/// yazılır ve boşaltılır, ardından `fs::rename` ile asıl dosyanın üzerine
/// taşınır. Böylece yazma sırasında bir çökme olursa eski depo bozulmaz.
pub fn kaydet(depo: &Depo, yol: &Path) -> Sonuc<()> {
    depo.dogrula()?;
    if let Some(ust) = yol.parent() {
        if !ust.as_os_str().is_empty() {
            fs::create_dir_all(ust)
                .map_err(|kaynak| Hata::io("depo dizini oluşturulamadı", ust, kaynak))?;
        }
    }
    let metin = serde_json::to_string_pretty(depo)
        .map_err(|kaynak| Hata::json("depo serileştirilemedi", kaynak))?;

    let gecici = gecici_yol(yol);
    {
        let mut dosya = fs::File::create(&gecici)
            .map_err(|kaynak| Hata::io("geçici dosya oluşturulamadı", &gecici, kaynak))?;
        dosya
            .write_all(metin.as_bytes())
            .map_err(|kaynak| Hata::io("depo yazılamadı", &gecici, kaynak))?;
        dosya
            .write_all(b"\n")
            .map_err(|kaynak| Hata::io("depo yazılamadı", &gecici, kaynak))?;
        dosya
            .sync_all()
            .map_err(|kaynak| Hata::io("depo diske yazılamadı", &gecici, kaynak))?;
    }
    fs::rename(&gecici, yol).map_err(|kaynak| Hata::io("depo değiştirilemedi", yol, kaynak))?;
    Ok(())
}

/// Depoyu yükler; dosya yoksa boş bir depo döner ve `false` ile bildirir.
pub fn yukle_veya_bos(yol: &Path) -> Sonuc<(Depo, bool)> {
    match fs::metadata(yol) {
        Ok(_) => Ok((yukle(yol)?, false)),
        Err(kaynak) if kaynak.kind() == std::io::ErrorKind::NotFound => Ok((Depo::bos(), true)),
        Err(kaynak) => Err(Hata::io("depo okunamadı", yol, kaynak)),
    }
}

/// Depo dosyasının biçim sürümü — dışa aktarım başlıklarında kullanılır.
pub fn surum() -> u32 {
    DEPO_SURUMU
}

/// Geçici dosya yolunu üretir: hedefin yanına, süreç kimliğiyle ayrışır.
pub fn gecici_yol(yol: &Path) -> PathBuf {
    let ad = yol
        .file_name()
        .map(|a| a.to_string_lossy().to_string())
        .unwrap_or_else(|| VARSAYILAN_DEPO_ADI.to_string());
    let dizin = yol.parent().unwrap_or_else(|| Path::new("."));
    dizin.join(format!("{ad}.{}.tmp", std::process::id()))
}

/// Depo yolunu çözer: bayrak > yürütülebilirin yanı > varsayılan ad.
pub fn yol_coz(bayrak: Option<&Path>, varsayilan_dizin: &Path) -> PathBuf {
    match bayrak {
        Some(p) => p.to_path_buf(),
        None => varsayilan_dizin.join(VARSAYILAN_DEPO_ADI),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Parca;
    use std::path::PathBuf;

    /// Test içinde geçici dizin üreten, `Drop` ile temizleyen kapsayıcı.
    ///
    /// Neden `tempfile` yok: bağımlılık politikası (WORKER_CONTRACT § 3.2)
    /// `tempfile`'i hiçbir projede vermez; yardımcı kendi kodumuzla yazılır.
    struct GeciciDizin {
        yol: PathBuf,
    }

    impl GeciciDizin {
        fn yeni(etiket: &str) -> std::io::Result<Self> {
            let kok = std::env::temp_dir().join(format!("sniphub-{etiket}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&kok);
            std::fs::create_dir_all(&kok)?;
            Ok(Self { yol: kok })
        }

        fn yol(&self) -> &Path {
            &self.yol
        }
    }

    impl Drop for GeciciDizin {
        fn drop(&mut self) {
            // Temizlik başarısız olsa da testi düşürmemeli; `let _ =`
            // bilinçlidir (WORKER_CONTRACT § 5.3).
            let _ = std::fs::remove_dir_all(&self.yol);
        }
    }

    fn ornek_depo() -> Depo {
        let mut depo = Depo::bos();
        depo.ekle(Parca::yeni(
            "Test parçası",
            ";t",
            "açıklama",
            crate::dil::Dil::Rust,
            vec!["test".into()],
            "fn main() {}",
            &crate::zaman::An::epoch_saniye(0),
        ))
        .expect("ekleme");
        depo
    }

    #[test]
    fn yaz_ve_oku_gidis_donusus_yapar() {
        let dizin = GeciciDizin::yeni("gidis-donus").expect("gecici dizin");
        let yol = dizin.yol().join("depo.json");
        let depo = ornek_depo();
        kaydet(&depo, &yol).expect("yazma");
        let okunan = yukle(&yol).expect("okuma");
        assert_eq!(depo, okunan);
    }

    #[test]
    fn yazma_atomiktir_ve_gecici_dosya_kalmaz() {
        let dizin = GeciciDizin::yeni("atomik").expect("gecici dizin");
        let yol = dizin.yol().join("depo.json");
        kaydet(&ornek_depo(), &yol).expect("yazma");
        let gecici = gecici_yol(&yol);
        assert!(!gecici.exists(), "geçici dosya kalmamalı");
        assert!(yol.exists());
    }

    #[test]
    fn bozuk_depo_dosyasi_hata_verir() {
        let dizin = GeciciDizin::yeni("bozuk").expect("gecici dizin");
        let yol = dizin.yol().join("bozuk.json");
        fs::write(&yol, "{ bu json değil ").expect("yazma");
        let hata = yukle(&yol).expect_err("bozuk dosya hata vermeli");
        assert!(matches!(hata, Hata::Json { .. }));
    }

    #[test]
    fn bos_dosya_hata_verir() {
        let dizin = GeciciDizin::yeni("bos").expect("gecici dizin");
        let yol = dizin.yol().join("bos.json");
        fs::write(&yol, "").expect("yazma");
        assert!(matches!(yukle(&yol), Err(Hata::BozukDepo { .. })));
    }

    #[test]
    fn desteklenmeyen_surum_hata_verir() {
        let dizin = GeciciDizin::yeni("surum").expect("gecici dizin");
        let yol = dizin.yol().join("surum.json");
        fs::write(&yol, r#"{"surum": 7, "parcalar": []}"#).expect("yazma");
        let hata = yukle(&yol).expect_err("sürüm hatası");
        assert!(matches!(hata, Hata::DesteklenmeyenSurum { bulunan: 7, .. }));
    }

    #[test]
    fn olmayan_depo_bos_donulur() {
        let dizin = GeciciDizin::yeni("yok").expect("gecici dizin");
        let yol = dizin.yol().join("yok.json");
        let (depo, yeni_mi) = yukle_veya_bos(&yol).expect("bos depo");
        assert!(yeni_mi);
        assert!(depo.parcalar.is_empty());
    }

    #[test]
    fn yol_cozme_onceliklari() {
        let bayrak = PathBuf::from("/tmp/bayrak.json");
        assert_eq!(
            yol_coz(Some(&bayrak), Path::new("/uygulama")),
            bayrak,
            "bayrak kazanır"
        );
        assert_eq!(
            yol_coz(None, Path::new("/uygulama")),
            PathBuf::from("/uygulama/parcalar.json")
        );
    }

    #[test]
    fn yazma_bozuk_schemayi_reddeder() {
        let dizin = GeciciDizin::yeni("sema").expect("gecici dizin");
        let yol = dizin.yol().join("sema.json");
        let mut depo = ornek_depo();
        depo.parcalar[0].govde = String::new();
        assert!(kaydet(&depo, &yol).is_err());
        assert!(!yol.exists(), "geçersiz depo diske yazılmamalı");
    }
}
