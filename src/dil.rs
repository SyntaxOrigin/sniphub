//! Dil etiketleri ve dosya uzantısı eşlemesi.
//!
//! Kapsam: yalnızca dil kimliği ve uzantı → dil normalleştirmesi. Sözdizimi
//! denetleme mantığı `syntax.rs` modülündedir; burada gramer bilgisi yoktur.
//!
//! 22 SnipHub / 25 TypeFast ayrımı (karar D-011) burada somutlaşır: `Metin`
//! çeşidi *programlama dili değildir* ve yapısal denetim kapsamı dışındadır.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Bir parçanın yazıldığı dil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Dil {
    /// Rust — yapısal denetleyici tam destekler.
    Rust,
    /// Python — yapısal denetleyici tam destekler.
    Python,
    /// JavaScript — uzantı çözümlemesi vardır, denetleyici desteklemez.
    JavaScript,
    /// SQL sorgusu — uzantı çözümlemesi vardır, denetleyici desteklemez.
    Sql,
    /// JSON — uzantı çözümlemesi vardır, denetleyici desteklemez.
    Json,
    /// TOML yapılandırması — uzantı çözümlemesi vardır, denetleyici desteklemez.
    Toml,
    /// Markdown — uzantı çözümlemesi vardır, denetleyici desteklemez.
    Markdown,
    /// Programlama dili değil, düz metin. 25 TypeFast'ın alanıdır.
    Metin,
}

impl Dil {
    /// Denetleyicinin bu dil için yapısal kontrol yapıp yapmadığı.
    pub const fn denetlenebilir(self) -> bool {
        matches!(self, Dil::Rust | Dil::Python)
    }

    /// Etiketin depo dosyasında ve komut satırında kullanılan kanonik yazımı.
    pub const fn etiket(self) -> &'static str {
        match self {
            Dil::Rust => "rust",
            Dil::Python => "python",
            Dil::JavaScript => "javascript",
            Dil::Sql => "sql",
            Dil::Json => "json",
            Dil::Toml => "toml",
            Dil::Markdown => "markdown",
            Dil::Metin => "metin",
        }
    }

    /// İnsan okuyan çıktılarda kullanılan görünen ad.
    pub const fn gorunen_ad(self) -> &'static str {
        match self {
            Dil::Rust => "Rust",
            Dil::Python => "Python",
            Dil::JavaScript => "JavaScript",
            Dil::Sql => "SQL",
            Dil::Json => "JSON",
            Dil::Toml => "TOML",
            Dil::Markdown => "Markdown",
            Dil::Metin => "Metin",
        }
    }

    /// Bir dosya uzantısından dil çözümler (nokta yoksa da kabul edilir).
    pub fn uzantiyla(uzanti: &str) -> Option<Dil> {
        let kucuk = uzanti.trim_start_matches('.').to_ascii_lowercase();
        match kucuk.as_str() {
            "rs" => Some(Dil::Rust),
            "py" | "pyi" => Some(Dil::Python),
            "js" | "mjs" | "cjs" | "ts" | "tsx" => Some(Dil::JavaScript),
            "sql" => Some(Dil::Sql),
            "json" => Some(Dil::Json),
            "toml" => Some(Dil::Toml),
            "md" | "markdown" => Some(Dil::Markdown),
            "txt" | "text" => Some(Dil::Metin),
            _ => None,
        }
    }

    /// Serbest metinden (komut satırı, README, içe aktarılan paket) dili çözümler.
    pub fn ayikla(metin: &str) -> Option<Dil> {
        let kucuk = metin.trim().to_ascii_lowercase();
        match kucuk.as_str() {
            "rust" | "rs" => Some(Dil::Rust),
            "python" | "py" => Some(Dil::Python),
            "javascript" | "js" | "ts" | "typescript" => Some(Dil::JavaScript),
            "sql" => Some(Dil::Sql),
            "json" => Some(Dil::Json),
            "toml" => Some(Dil::Toml),
            "markdown" | "md" => Some(Dil::Markdown),
            "metin" | "text" | "txt" | "plain" => Some(Dil::Metin),
            _ => None,
        }
    }
}

impl fmt::Display for Dil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.etiket())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uzantilar_dil_cozer() {
        assert_eq!(Dil::uzantiyla("rs"), Some(Dil::Rust));
        assert_eq!(Dil::uzantiyla(".PY"), Some(Dil::Python));
        assert_eq!(Dil::uzantiyla("pyi"), Some(Dil::Python));
        assert_eq!(Dil::uzantiyla("ts"), Some(Dil::JavaScript));
        assert_eq!(Dil::uzantiyla("sql"), Some(Dil::Sql));
        assert_eq!(Dil::uzantiyla("md"), Some(Dil::Markdown));
        assert_eq!(Dil::uzantiyla("zzz"), None);
    }

    #[test]
    fn serbest_metin_dil_cozer() {
        assert_eq!(Dil::ayikla("Rust"), Some(Dil::Rust));
        assert_eq!(Dil::ayikla("  python "), Some(Dil::Python));
        assert_eq!(Dil::ayikla("bilinmeyen"), None);
    }

    #[test]
    fn yalnizca_rust_ve_python_denetlenir() {
        assert!(Dil::Rust.denetlenebilir());
        assert!(Dil::Python.denetlenebilir());
        assert!(!Dil::Sql.denetlenebilir());
        assert!(!Dil::Metin.denetlenebilir());
    }

    #[test]
    fn metin_dili_programlama_dili_degildir() {
        // D-011: düz metin denetleyici kapsamı dışıdır, bu bir kural değil tesadüftür.
        assert_eq!(Dil::Metin.etiket(), "metin");
        assert!(!Dil::Metin.denetlenebilir());
    }

    #[test]
    fn gorunen_ad_etiketi_kapsar() {
        for dil in [
            Dil::Rust,
            Dil::Python,
            Dil::JavaScript,
            Dil::Sql,
            Dil::Json,
            Dil::Toml,
            Dil::Markdown,
            Dil::Metin,
        ] {
            assert!(dil
                .etiket()
                .contains(&dil.gorunen_ad().to_ascii_lowercase()[..2]));
        }
    }
}
