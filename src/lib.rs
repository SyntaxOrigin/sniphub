//! # SnipHub — programlama parçaları yöneticisi
//!
//! SnipHub, `tetik → kod` genişletmesi yapan bir **programlama** parçası
//! yöneticisidir: parçalar bir dil etiketiyle saklanır, yapısal sözdizimi
//! denetiminden geçirilir ve çoklu dil aramasıyla bulunur.
//!
//! ## 22 SnipHub / 25 TypeFast ayrımı (karar D-011)
//!
//! Bu kütüphane **programlama** içindir. Bir özellik yalnızca "hangi dilde
//! yazıldığı" bilgisine bağlıysa SnipHub'a aittir. Genel metin şablonları,
//! pano geçmişi, gizli mod ve genel kısayol sözlüğü **bilinçli olarak
//! uygulanmamıştır**; bunlar kardeş proje 25 TypeFast'ın alanıdır.
//!
//! ## Güvenlik uyarısı
//!
//! Parça deposu **düz metindir**; şifreleme yoktur. Argon2id + XChaCha20
//! kripto crate'leri WORKER_CONTRACT.md § 3.2-C uyarınca yalnızca 16/17/30
//! projelerine serbest olduğu için bu projede kullanılamaz. Depoya parola,
//! anahtar veya kimlik bilgisi koymayın.
//!
//! ## Modüller
//!
//! | Modül | Sorumluluk |
//! |---|---|
//! | [`hata`] | Tek hata tipi ve `Display`/`Error` uygulamaları |
//! | [`zaman`] | `std::time` tabanlı ISO-8601 zaman damgaları |
//! | [`dil`] | Dil etiketleri ve uzantı → dil çözümlemesi |
//! | [`model`] | `Parca` / `Depo` şeması ve saf iş kuralları |
//! | [`genisle`] | `{{belirtec}}` genişleme dili |
//! | [`syntax`] | Rust ve Python için yapısal denge denetleyicisi |
//! | [`depo`] | Atomik okuma/yazma, biçim sürümü |
//! | [`arama`] | Çoklu dil arama, filtre ve alaka sıralaması |
//! | [`kazanc`] | Fiziksel tuş sayacı tabanlı kazanç hesabı |
//! | [`aktarma`] | JSON / Markdown / HTML dışa-içe aktarma |
//! | [`istatistik`] | Kullanım ve kazanç raporu |

#![forbid(unsafe_code)]
#![deny(missing_docs)]
// Üretim kodunda `unwrap`/`expect` kullanılamaz; testler bunları meşru olarak
// kullanır, bu yüzden lint yalnızca test dışı derlemede etkinleşir.
#![cfg_attr(not(test), warn(clippy::unwrap_used, clippy::expect_used))]

pub mod aktarma;
pub mod arama;
pub mod depo;
pub mod dil;
pub mod genisle;
pub mod hata;
pub mod istatistik;
pub mod kazanc;
pub mod model;
pub mod syntax;
pub mod zaman;

pub use aktarma::{Bicim, Cakisma, Ozet};
pub use arama::Sorgu;
pub use dil::Dil;
pub use genisle::{Genisleme, Genisletici};
pub use hata::{Hata, Sonuc};
pub use istatistik::Istatistik;
pub use kazanc::{Ayarlar, Kazanc};
pub use model::{Depo, Dogrulama, Parca};
pub use zaman::An;

/// Araç sürümü; `Cargo.toml` ile eşleşmelidir.
pub const SURUM: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kutuphane_kamu_api_yuzeyini_disa_aktarir() {
        // Alt komutların kullandığı temel türler buradan erişilebilir olmalı.
        let _: fn(&str) -> Result<Genisleme, Hata>;
        let _: fn(Dil, &str) -> syntax::Denetim = syntax::denetle;
    }

    #[test]
    fn surum_paket_versiyonuyla_ayni() {
        assert_eq!(SURUM, "0.1.0");
    }
}
