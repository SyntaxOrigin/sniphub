//! Zaman damgası üretimi.
//!
//! Kapsam: `chrono`/`time` bağımlılıkları yasak (WORKER_CONTRACT.md § 3.2-F),
//! bu yüzden ISO-8601 dönüşümü Howard Hinnant'ın "civil_from_days /
//! days_from_civil" algoritmasıyla elle yazılmıştır. Algoritma saf bir
//! tamsayı dönüşümüdür ve testlerde sabit tarihlerle doğrulanır; sistem
//! saatinin test kararlarında kullanılmasına gerek kalmaz.

use std::time::{SystemTime, UNIX_EPOCH};

/// Bugünün Unix epoch (1970-01-01) üzerinden geçen saniye sayısı.
///
/// Hata durumunda `0` döner; tarih alanları bilgilendirme amaçlı olduğu için
/// bu, aracı çökertmemek adına bilinçli bir seçimdir.
pub fn simdi_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|s| s.as_secs())
        .unwrap_or(0)
}

/// Gün sayısını (epoch tabanlı) `(yil, ay, gun)` sivil tarihine çevirir.
///
/// Hinnant algoritması: <https://howardhinnant.github.io/date_algorithms.html>
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Sivil `(yıl, ay, gün)` tarihini Unix epoch saniyesine çevirir.
///
/// UTC gün başı 00:00:00 kabul edilir. Gün/hata doğrulaması yapılmaz;
/// geçersiz bir tarih modülü hatası değil, çağıranın hatasıdır.
pub fn epoch_saniye(yil: i64, ay: u32, gun: u32) -> u64 {
    let gunlar = days_from_civil(yil, ay, gun);
    if gunlar < 0 {
        return 0;
    }
    (gunlar as u64).saturating_mul(86_400)
}

/// Sivil `(yıl, ay, gün)` tarihini epoch tabanlı gün sayısına çevirir.
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = if m > 2 { m - 3 } else { m + 9 } as i64; // [0, 11]
    let doy = (153 * mp + 2) / 5 + d as i64 - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

/// Unix epoch saniyesini `YYYY-MM-DD` biçiminde UTC tarihe çevirir.
pub fn iso_tarih(epoch: u64) -> String {
    let gun = (epoch / 86_400) as i64;
    let (y, a, g) = civil_from_days(gun);
    format!("{y:04}-{a:02}-{g:02}")
}

/// Unix epoch saniyesini `HH:MM:SS` biçiminde UTC saate çevirir.
pub fn iso_saat(epoch: u64) -> String {
    let kalan = epoch % 86_400;
    format!(
        "{:02}:{:02}:{:02}",
        kalan / 3600,
        (kalan % 3600) / 60,
        kalan % 60
    )
}

/// Unix epoch saniyesini `YYYY-MM-DDTHH:MM:SSZ` biçimine çevirir.
pub fn iso_zaman(epoch: u64) -> String {
    format!("{}T{}Z", iso_tarih(epoch), iso_saat(epoch))
}

/// Yerleşik `{{tarih}}`, `{{saat}}` ve `{{zaman}}` belirteçlerinin
/// üretilmesinde kullanılan anlık görüntü.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct An {
    /// ISO-8601 UTC tarih damgası.
    pub tarih: String,
    /// `HH:MM:SS` biçiminde saat.
    pub saat: String,
    /// `YYYY-MM-DDTHH:MM:SSZ` biçiminde tam zaman damgası.
    pub zaman: String,
}

impl An {
    /// Verilen epoch saniyesinden bir zaman görüntüsü üretir.
    pub fn epoch_saniye(epoch: u64) -> Self {
        An {
            tarih: iso_tarih(epoch),
            saat: iso_saat(epoch),
            zaman: iso_zaman(epoch),
        }
    }

    /// Sistem saatinden bir zaman görüntüsü üretir.
    pub fn simdi() -> Self {
        An::epoch_saniye(simdi_epoch())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_tarihi_bozulmaz() {
        assert_eq!(iso_tarih(0), "1970-01-01");
        assert_eq!(iso_saat(0), "00:00:00");
        assert_eq!(iso_zaman(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn bilinen_tarihler_dogru_cevirilir() {
        // 2000-03-01T00:00:00Z, 951868800
        assert_eq!(iso_zaman(951_868_800), "2000-03-01T00:00:00Z");
        // 2026-09-29T12:34:56Z — proje tarihi ile eşleşen sabit vektör
        assert_eq!(iso_zaman(1_790_685_296), "2026-09-29T12:34:56Z");
        // 2000-02-29 — artık yıl sınırı
        assert_eq!(iso_tarih(951_782_400), "2000-02-29");
    }

    #[test]
    fn gun_duzeni_geri_donusur() {
        for &(y, a, g) in &[
            (1970, 1, 1),
            (1999, 12, 31),
            (2000, 2, 29),
            (2024, 2, 29),
            (2026, 9, 29),
            (2100, 3, 1),
        ] {
            let gun = days_from_civil(y, a, g);
            assert_eq!(civil_from_days(gun), (y, a, g), "{y}-{a}-{g} gidiş-dönüş");
        }
    }

    #[test]
    fn epoch_saniye_tarih_donusturur() {
        assert_eq!(epoch_saniye(1970, 1, 1), 0);
        assert_eq!(epoch_saniye(1970, 1, 2), 86_400);
        assert_eq!(iso_zaman(epoch_saniye(2026, 9, 29)), "2026-09-29T00:00:00Z");
        assert_eq!(
            epoch_saniye(1969, 12, 31),
            0,
            "epoch öncesi tarih 0'a çıkarılır"
        );
    }

    #[test]
    fn saniye_kalani_dogru_hesaplanir() {
        assert_eq!(iso_saat(86_399), "23:59:59");
        assert_eq!(iso_saat(86_400), "00:00:00");
        assert_eq!(iso_tarih(86_400), "1970-01-02");
    }
}
