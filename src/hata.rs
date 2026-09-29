//! Merkezî hata tipi.
//!
//! Bu modül `thiserror` gibi bir türetme makrosu **kullanamaz** (bağımlılık
//! politikası: WORKER_CONTRACT.md § 3.2-F), bu yüzden `Display` ve
//! `Error` uygulamaları elle yazılmıştır.
//!
//! Kapsam: yalnızca SnipHub'ın kendi hataları. `std::io::Error` ve
//! `serde_json::Error` bu enum'ün içinde taşınır, böylece çağıran taraf tek
//! bir hata tipiyle çalışır.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// SnipHub'ın tüm geri dönüş değerlerinde kullandığı hata tipi.
#[derive(Debug)]
pub enum Hata {
    /// Dosya sistemi işlemi başarısız oldu.
    Io {
        /// Başarısız olan işlemin kısa adı (hata mesajında kullanılır).
        islem: &'static str,
        /// İşlemin yürütüldüğü yol.
        yol: PathBuf,
        /// Altındaki işletim sistemi hatası.
        kaynak: io::Error,
    },
    /// JSON çözümleme/serileştirme hatası.
    Json {
        /// Hatanın bağlamı ("depo okunamadı", "dışa aktarım" gibi).
        baglam: &'static str,
        /// Altındaki `serde_json` hatası.
        kaynak: serde_json::Error,
    },
    /// Depo dosyasının biçim sürümü desteklenmiyor.
    DesteklenmeyenSurum {
        /// Dosyada bulunan sürüm.
        bulunan: u32,
        /// Programın anladığı sürüm.
        desteklenen: u32,
    },
    /// Depo dosyası JSON olarak çözümlendi ama şema denetiminden geçemedi.
    BozukDepo {
        /// Denetimi ihlal eden alanın yolu (ör. `parcalar[2].ad`).
        alan: String,
        /// İhlalin açıklaması.
        neden: String,
    },
    /// Aynı adda ikinci bir parça eklenmek istendi.
    YinelenenAd {
        /// Çakışan parçanın adı.
        ad: String,
    },
    /// Aranan parça depoda bulunamadı.
    ParcaYok {
        /// Aranan ad veya tetik.
        anahtar: String,
    },
    /// `{{` açılışı eşleşen `}}` ile kapatılmamış.
    KapanmamisBelirtec {
        /// Açılışın 1 tabanlı satırı.
        satir: usize,
        /// Açılışın 1 tabanlı sütunu.
        sutun: usize,
    },
    /// Eşleşmeyen bir `}}` görüldü.
    AsiriKapanis {
        /// Kapanışın 1 tabanlı satırı.
        satir: usize,
        /// Kapanışın 1 tabanlı sütunu.
        sutun: usize,
    },
    /// Belirteç gövdesi geçersiz (alan sayısı, ad biçimi, indeks).
    GecersizBelirtec {
        /// Belirtecin ham içeriği (`{{` ve `}}` hariç).
        metin: String,
        /// Satır (1 tabanlı).
        satir: usize,
        /// Sütun (1 tabanlı).
        sutun: usize,
        /// Reddedilme nedeni.
        neden: &'static str,
    },
    /// Genişleme adımı tavanı aşıldı (sonsuz döngü koruması).
    AdimTavani {
        /// Uygulanan tavan.
        tavan: usize,
    },
    /// Değer atamasında `=` yok ya da anahtar boş.
    GecersizDeger {
        /// Kullanıcının verdiği ham `anahtar=değer` metni.
        ham: String,
    },
    /// Bilinmeyen dil etiketi.
    BilinmeyenDil {
        /// Kullanıcının verdiği ham etiket.
        etiket: String,
    },
    /// Yanlış dışa/içe aktarma biçimi.
    DesteklenmeyenBicim {
        /// İstenen biçim adı.
        bicim: String,
    },
    /// HTML çıktısında kaçırılamayan bir yapısal hata oluştu.
    BozukCikti {
        /// Açıklama.
        neden: &'static str,
    },
}

impl fmt::Display for Hata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Hata::Io { islem, yol, kaynak } => {
                write!(f, "{islem} işlemi başarısız ({}): {kaynak}", yol.display())
            }
            Hata::Json { baglam, kaynak } => write!(f, "{baglam}: {kaynak}"),
            Hata::DesteklenmeyenSurum {
                bulunan,
                desteklenen,
            } => write!(
                f,
                "desteklenmeyen depo sürümü {bulunan} (bu sürüm {desteklenen} sürümünü okur)"
            ),
            Hata::BozukDepo { alan, neden } => write!(f, "bozuk depo: {alan} — {neden}"),
            Hata::YinelenenAd { ad } => write!(f, "bu adda bir parça zaten var: {ad}"),
            Hata::ParcaYok { anahtar } => write!(f, "parça bulunamadı: {anahtar}"),
            Hata::KapanmamisBelirtec { satir, sutun } => write!(
                f,
                "satır {satir}, sütun {sutun}: `{{{{` açılışı `}}}}` ile kapatılmamış"
            ),
            Hata::AsiriKapanis { satir, sutun } => {
                write!(f, "satır {satir}, sütun {sutun}: eşleşmeyen `}}}}`")
            }
            Hata::GecersizBelirtec {
                metin,
                satir,
                sutun,
                neden,
            } => write!(
                f,
                "satır {satir}, sütun {sutun}: geçersiz belirteç `{{{{{metin}}}}}` — {neden}"
            ),
            Hata::AdimTavani { tavan } => {
                write!(f, "genişleme adımı tavanı ({tavan}) aşıldı; döngü koruması")
            }
            Hata::GecersizDeger { ham } => write!(
                f,
                "geçersiz değer ataması: `{ham}` (beklenen: anahtar=değer)"
            ),
            Hata::BilinmeyenDil { etiket } => write!(f, "bilinmeyen dil etiketi: `{etiket}`"),
            Hata::DesteklenmeyenBicim { bicim } => write!(f, "desteklenmeyen biçim: `{bicim}`"),
            Hata::BozukCikti { neden } => write!(f, "çıktı üretilemedi: {neden}"),
        }
    }
}

impl Error for Hata {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Hata::Io { kaynak, .. } => Some(kaynak),
            Hata::Json { kaynak, .. } => Some(kaynak),
            _ => None,
        }
    }
}

/// SnipHub'ın kısa yolu: `Result<T, Hata>`.
pub type Sonuc<T> = Result<T, Hata>;

impl Hata {
    /// `io::Error` ve yolu `Hata::Io` sarmalayıcısına çevirir.
    pub fn io(islem: &'static str, yol: impl Into<PathBuf>, kaynak: io::Error) -> Self {
        Hata::Io {
            islem,
            yol: yol.into(),
            kaynak,
        }
    }

    /// `serde_json::Error` ve bağlamı `Hata::Json` sarmalayıcısına çevirir.
    pub fn json(baglam: &'static str, kaynak: serde_json::Error) -> Self {
        Hata::Json { baglam, kaynak }
    }
}
