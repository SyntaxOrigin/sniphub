//! Dışa ve içe aktarma: JSON, Markdown ve tek dosya HTML katalog.
//!
//! Kapsam: üç çıktı biçiminin üretimi ve içe aktarma çakışma politikası.
//! HTML katalog **tek dosyadır**: hiçbir dış kaynak (CSS, JS, yazı tipi)
//! referanslanmaz, bu yüzden çevrimdışı ve USB'den çalışır.
//!
//! Markdown biçimi **gidiş-dönüşlüdür**: `aktarma::markdown_uret` çıktısı
//! `aktarma::markdown_oku` ile aynı parça listesine çözülür.

use crate::dil::Dil;
use crate::hata::{Hata, Sonuc};
use crate::model::{Depo, Parca, DEPO_SURUMU};
use crate::syntax;
use crate::zaman::An;
use std::fmt::Write as _;

/// Dışa aktarım biçimi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bicim {
    /// Makine okunur tam depo şeması.
    Json,
    /// İnsan okunur Markdown katalog.
    Markdown,
    /// Tek dosya HTML katalog.
    Html,
}

impl Bicim {
    /// Komut satırındaki adı.
    pub const fn ad(self) -> &'static str {
        match self {
            Bicim::Json => "json",
            Bicim::Markdown => "markdown",
            Bicim::Html => "html",
        }
    }

    /// Serbest metinden biçim çözer.
    pub fn ayikla(metin: &str) -> Sonuc<Bicim> {
        match metin.trim().to_ascii_lowercase().as_str() {
            "json" => Ok(Bicim::Json),
            "markdown" | "md" => Ok(Bicim::Markdown),
            "html" => Ok(Bicim::Html),
            diger => Err(Hata::DesteklenmeyenBicim {
                bicim: diger.to_string(),
            }),
        }
    }

    /// Dosya uzantısından biçim çözer.
    pub fn uzantidan(uzanti: &str) -> Sonuc<Bicim> {
        Bicim::ayikla(uzanti)
    }
}

/// İçe aktarmada aynı adda bir parça bulunduğunda uygulanacak kural.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cakisma {
    /// İçe aktar mayı durdur, hata döner.
    Hata,
    /// Var olan parçayı korur.
    Atla,
    /// Var olan parçayı gelenle değiştirir.
    Degistir,
    /// Gelen parçayı `-2`, `-3` … son ekine alarak ekler.
    YenidenAdlandir,
}

impl Cakisma {
    /// Komut satırındaki adı.
    pub const fn ad(self) -> &'static str {
        match self {
            Cakisma::Hata => "hata",
            Cakisma::Atla => "atla",
            Cakisma::Degistir => "degistir",
            Cakisma::YenidenAdlandir => "yeniden-adlandir",
        }
    }

    /// Serbest metinden politika çözer.
    pub fn ayikla(metin: &str) -> Sonuc<Cakisma> {
        match metin.trim().to_ascii_lowercase().as_str() {
            "hata" | "error" => Ok(Cakisma::Hata),
            "atla" | "skip" => Ok(Cakisma::Atla),
            "degistir" | "replace" => Ok(Cakisma::Degistir),
            "yeniden-adlandir" | "rename" => Ok(Cakisma::YenidenAdlandir),
            diger => Err(Hata::DesteklenmeyenBicim {
                bicim: diger.to_string(),
            }),
        }
    }
}

/// Bir içe aktarma işleminin sonucu.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ozet {
    /// Yeni eklenen parça sayısı.
    pub eklenen: usize,
    /// Var olan parça korunduğu için atlanan sayısı.
    pub atlanan: usize,
    /// Var olan parçanın yerine geçen sayısı.
    pub degisen: usize,
    /// Yeniden adlandırılarak eklenen sayısı.
    pub yeniden_adlandirilan: usize,
}

impl Ozet {
    /// Hiçbir değişiklik olmadı mı.
    pub fn degisiklik_yok(&self) -> bool {
        self.eklenen == 0 && self.degisen == 0 && self.yeniden_adlandirilan == 0
    }
}

/// Depoyu seçilen biçimde dışa aktarır.
pub fn disa_aktar(depo: &Depo, bicim: Bicim) -> Sonuc<String> {
    match bicim {
        Bicim::Json => serde_json::to_string_pretty(depo)
            .map_err(|kaynak| Hata::json("JSON dışa aktarımı başarısız", kaynak)),
        Bicim::Markdown => Ok(markdown_uret(depo)),
        Bicim::Html => Ok(html_uret(depo)),
    }
}

/// JSON dışa aktarımı (okunabilir biçimlendirme).
pub fn json_uret(depo: &Depo) -> Sonuc<String> {
    serde_json::to_string_pretty(depo)
        .map_err(|kaynak| Hata::json("JSON dışa aktarımı başarısız", kaynak))
}

/// Markdown katalog üretir (gidiş-dönüşlü).
pub fn markdown_uret(depo: &Depo) -> String {
    let mut c = String::new();
    let _ = writeln!(c, "# SnipHub Katalogu");
    let _ = writeln!(c);
    let _ = writeln!(
        c,
        "<!-- surum: {DEPO_SURUMU} | parca: {} -->",
        depo.parcalar.len()
    );
    let _ = writeln!(c);
    for parca in &depo.parcalar {
        let _ = writeln!(c, "## `{}`", parca.ad);
        let _ = writeln!(c);
        let _ = writeln!(
            c,
            "- tetik: `{}`\n- dil: `{}`\n- etiket: `{}`\n- kullanim: `{}`\n- dogrulama: `{}`",
            parca.tetik,
            parca.dil.etiket(),
            parca.etiketler.join(", "),
            parca.kullanim_sayaci,
            parca.dogrulama.etiket()
        );
        let _ = writeln!(c);
        if !parca.aciklama.is_empty() {
            let _ = writeln!(c, "> {}", parca.aciklama);
            let _ = writeln!(c);
        }
        let _ = writeln!(c, "```{}", parca.dil.etiket());
        let _ = writeln!(c, "{}", parca.govde);
        let _ = writeln!(c, "{}", cit_kapat(&parca.govde));
        let _ = writeln!(c);
    }
    c
}

/// Gövdenin kendi içinde bulunan en uzun geri tik dizisinden uzun bir çit
/// seçer; aksi hâlde gövde içindeki bir çit katalogun sonunu erken kapatır.
fn cit_kapat(govde: &str) -> String {
    "`".repeat((enz_uzun_geri_tik_dizisi(govde) + 1).max(3))
}

/// Metindeki en uzun ardışık geri tik dizisinin uzunluğu.
fn enz_uzun_geri_tik_dizisi(metin: &str) -> usize {
    let mut en_uzun = 0usize;
    let mut sayac = 0usize;
    for ch in metin.chars() {
        if ch == '`' {
            sayac += 1;
            en_uzun = en_uzun.max(sayac);
        } else {
            sayac = 0;
        }
    }
    en_uzun
}

/// Markdown katalogu ayrıştırıp depo listesine çevirir.
pub fn markdown_oku(metin: &str) -> Sonuc<Vec<Parca>> {
    let mut parcalar: Vec<Parca> = Vec::new();
    let mut ad: Option<String> = None;
    let mut alanlar: Vec<(String, String)> = Vec::new();
    let mut aciklama = String::new();
    let mut govde: Option<String> = None;
    let mut govde_dili = Dil::Metin;
    let mut govde_cit_uzunlugu = 3usize;
    let an = An::epoch_saniye(0);

    for satir in metin.lines() {
        let kirp = satir.trim_end();
        if let Some(kalan) = kirp.strip_prefix("## `") {
            if let Some(isim) = kalan.strip_suffix('`') {
                parca_bitir(
                    &mut parcalar,
                    &mut ad,
                    &mut alanlar,
                    &mut aciklama,
                    &mut govde,
                    govde_dili,
                    &an,
                );
                ad = Some(isim.to_string());
                continue;
            }
        }
        if ad.is_some() && govde.is_none() {
            if kirp.starts_with("```") {
                govde = Some(String::new());
                govde_dili = Dil::ayikla(kirp.trim_start_matches('`')).unwrap_or(Dil::Metin);
                govde_cit_uzunlugu = kirp.chars().filter(|c| *c == '`').count().max(3);
                continue;
            }
            if let Some(kalan) = kirp.strip_prefix("- ") {
                if let Some((anahtar, deger)) = kalan.split_once(':') {
                    alanlar.push((
                        anahtar.trim().to_string(),
                        deger.trim().trim_matches('`').to_string(),
                    ));
                }
                continue;
            }
            if let Some(kalan) = kirp.strip_prefix("> ") {
                aciklama = kalan.to_string();
                continue;
            }
        }
        if let Some(g) = &mut govde {
            // Yalnızca açılış çiti kadar (veya daha uzun) bir çit kapatır;
            // gövdenin içindeki daha kısa çitler gövde metnine aittir.
            if kirp.chars().all(|c| c == '`') && kirp.len() >= govde_cit_uzunlugu {
                parca_bitir(
                    &mut parcalar,
                    &mut ad,
                    &mut alanlar,
                    &mut aciklama,
                    &mut govde,
                    govde_dili,
                    &an,
                );
                continue;
            }
            g.push_str(kirp);
            g.push('\n');
        }
    }
    parca_bitir(
        &mut parcalar,
        &mut ad,
        &mut alanlar,
        &mut aciklama,
        &mut govde,
        govde_dili,
        &an,
    );
    parcalar.retain(|p| !p.govde.is_empty());
    Ok(parcalar)
}

/// Bir Markdown bölümünü kapatıp parça listesine ekler.
#[allow(clippy::too_many_arguments)]
fn parca_bitir(
    parcalar: &mut Vec<Parca>,
    ad: &mut Option<String>,
    alanlar: &mut Vec<(String, String)>,
    aciklama: &mut String,
    govde: &mut Option<String>,
    govde_dili: Dil,
    an: &An,
) {
    let (Some(isim), Some(govde_metni)) = (ad.clone(), govde.clone()) else {
        *ad = None;
        *alanlar = Vec::new();
        *aciklama = String::new();
        return;
    };
    let al = |anahtar: &str| -> Option<String> {
        alanlar
            .iter()
            .find(|(k, _)| k == anahtar)
            .map(|(_, v)| v.clone())
    };
    // Dil, kod çiti (```) dil etiketinden okunur; `- dil:` alanı yalnızca
    // insan okuması için yazılır ve çiti belirtmiyorsa o geçerlidir.
    let dil = if govde_dili == Dil::Metin {
        Dil::ayikla(&al("dil").unwrap_or_default()).unwrap_or(Dil::Metin)
    } else {
        govde_dili
    };
    let parca = Parca::yeni(
        &isim,
        &al("tetik").unwrap_or_default(),
        aciklama,
        dil,
        al("etiket")
            .map(|e| {
                e.split(',')
                    .map(|x| x.trim().to_string())
                    .filter(|x| !x.is_empty())
                    .collect()
            })
            .unwrap_or_default(),
        govde_metni.trim_end_matches('\n'),
        an,
    );
    parcalar.push(parca);
    *ad = None;
    *alanlar = Vec::new();
    *aciklama = String::new();
    *govde = None;
}

/// Tek dosya HTML katalog üretir.
pub fn html_uret(depo: &Depo) -> String {
    let mut c = String::with_capacity(4096);
    let _ = writeln!(c, "<!DOCTYPE html>");
    let _ = writeln!(c, "<html lang=\"tr\">");
    let _ = writeln!(c, "<head>");
    let _ = writeln!(c, "<meta charset=\"utf-8\">");
    let _ = writeln!(
        c,
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
    );
    let _ = writeln!(c, "<title>SnipHub Katalogu</title>");
    let _ = writeln!(c, "<style>");
    let _ = writeln!(c, "{}", HTML_STIL);
    let _ = writeln!(c, "</style>");
    let _ = writeln!(c, "</head>");
    let _ = writeln!(c, "<body>");
    let _ = writeln!(c, "<h1>SnipHub Katalogu</h1>");
    let _ = writeln!(
        c,
        "<p class=\"ozet\">{} parça · biçim sürümü {DEPO_SURUMU} · yapısal denge kontrolü raporlandı, gramer doğrulaması yapılmadı.</p>",
        depo.parcalar.len()
    );
    for parca in &depo.parcalar {
        let _ = writeln!(c, "<section class=\"parca\">");
        let _ = writeln!(c, "<h2>{}</h2>", kacis(&parca.ad));
        let _ = writeln!(
            c,
            "<p class=\"meta\"><code>{}</code> · {} · {} kullanım</p>",
            kacis(&parca.tetik),
            parca.dil.gorunen_ad(),
            parca.kullanim_sayaci
        );
        if !parca.etiketler.is_empty() {
            let etiketler: Vec<String> = parca
                .etiketler
                .iter()
                .map(|e| format!("<span class=\"etiket\">{}</span>", kacis(e)))
                .collect();
            let _ = writeln!(c, "<p class=\"etiketler\">{}</p>", etiketler.join(" "));
        }
        if !parca.aciklama.is_empty() {
            let _ = writeln!(c, "<p class=\"aciklama\">{}</p>", kacis(&parca.aciklama));
        }
        let _ = writeln!(c, "<pre><code>{}</code></pre>", kacis(&parca.govde));
        let denetim = syntax::denetle(parca.dil, &parca.govde);
        let (sinif, not) = if denetim.gecerli() {
            ("ok", "yapısal denge tamam")
        } else {
            ("hata", "yapısal denge tamam değil")
        };
        let _ = writeln!(c, "<p class=\"denetim {sinif}\">{not}</p>");
        let _ = writeln!(c, "</section>");
    }
    let _ = writeln!(c, "</body>");
    let _ = writeln!(c, "</html>");
    c
}

/// HTML katalogun gömülü stili (tek dosya koşulu).
const HTML_STIL: &str = ":root{color-scheme:light dark}\
body{font-family:system-ui,\"Segoe UI\",Roboto,sans-serif;margin:2rem auto;max-width:60rem;\
padding:0 1rem;line-height:1.6;background:#f5f7fa;color:#161b22}\
h1{border-bottom:2px solid #1f5fd0;padding-bottom:.4rem}\
.ozet,.meta,.aciklama{color:#57606d;font-size:.92rem}\
.parca{border:1px solid #d3dae3;border-radius:10px;padding:1rem 1.2rem;margin:1.2rem 0;background:#fff}\
pre{background:#f1f3f7;border-radius:8px;padding:.9rem 1rem;overflow-x:auto}\
code{font-family:ui-monospace,Consolas,monospace}\
.etiket{display:inline-block;background:#e7effc;color:#1f5fd0;border-radius:999px;\
padding:.1rem .6rem;font-size:.82rem;margin-right:.3rem}\
.denetim{font-size:.85rem}.denetim.ok{color:#14683c}.denetim.hata{color:#9d2626}\
@media(prefers-color-scheme:dark){body{background:#0e1116;color:#e6edf3}\
.parca{background:#161b22;border-color:#2b333d}pre{background:#10161d}\
.ozet,.meta,.aciklama{color:#9aa7b4}.etiket{background:#17233a;color:#6ea8fe}}";

/// HTML kaçışı: `<`, `>`, `&`, `"` ve `'` dışarı çevrilir.
pub fn kacis(metin: &str) -> String {
    let mut c = String::with_capacity(metin.len() + 16);
    for ch in metin.chars() {
        match ch {
            '&' => c.push_str("&amp;"),
            '<' => c.push_str("&lt;"),
            '>' => c.push_str("&gt;"),
            '"' => c.push_str("&quot;"),
            '\'' => c.push_str("&#39;"),
            _ => c.push(ch),
        }
    }
    c
}

/// Dışa aktarılmış metni depo listesine çevirir ve çakışma politikasını uygular.
pub fn ice_aktar(depo: &mut Depo, metin: &str, bicim: Bicim, politika: Cakisma) -> Sonuc<Ozet> {
    let gelenler = match bicim {
        Bicim::Json => {
            let yeni: Depo = serde_json::from_str(metin)
                .map_err(|kaynak| Hata::json("JSON içe aktarımı başarısız", kaynak))?;
            yeni.parcalar
        }
        Bicim::Markdown => markdown_oku(metin)?,
        Bicim::Html => {
            return Err(Hata::DesteklenmeyenBicim {
                bicim: "html (içe aktarım desteklenmiyor)".into(),
            })
        }
    };
    let mut ozet = Ozet::default();
    for parca in gelenler {
        uygula(depo, parca, politika, &mut ozet)?;
    }
    Ok(ozet)
}

/// Tek bir gelen parçayı çakışma politikasına göre uygular.
fn uygula(depo: &mut Depo, mut parca: Parca, politika: Cakisma, ozet: &mut Ozet) -> Sonuc<()> {
    let mevcut = depo.adla(&parca.ad).map(|p| p.id.clone());
    match mevcut {
        None => {
            depo.ekle(parca)?;
            ozet.eklenen += 1;
        }
        Some(id) => match politika {
            Cakisma::Hata => return Err(Hata::YinelenenAd { ad: parca.ad }),
            Cakisma::Atla => ozet.atlanan += 1,
            Cakisma::Degistir => {
                if let Some(hedef) = depo.parcalar.iter_mut().find(|p| p.id == id) {
                    parca.id = id;
                    *hedef = parca;
                }
                ozet.degisen += 1;
            }
            Cakisma::YenidenAdlandir => {
                parca.ad = benzersiz_ad(depo, &parca.ad);
                parca.id = Parca::kimlik_uret(&parca.ad, &parca.govde);
                depo.ekle(parca)?;
                ozet.yeniden_adlandirilan += 1;
            }
        },
    }
    Ok(())
}

/// Ad çakışmasını çözecek benzersiz ad üretir (`ad-2`, `ad-3`, …).
pub fn benzersiz_ad(depo: &Depo, ad: &str) -> String {
    let mut n = 2u32;
    let mut aday = format!("{ad}-{n}");
    while depo.adla(&aday).is_some() {
        n += 1;
        aday = format!("{ad}-{n}");
    }
    aday
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Dogrulama;
    use crate::zaman::An;

    fn parca(ad: &str, tetik: &str, govde: &str) -> Parca {
        Parca::yeni(
            ad,
            tetik,
            "açıklama",
            Dil::Rust,
            vec!["et".into()],
            govde,
            &An::epoch_saniye(0),
        )
    }

    fn depo_ornegi() -> Depo {
        let mut depo = Depo::bos();
        depo.ekle(parca("Bir", ";a", "fn a() {}")).expect("ekleme");
        depo.ekle(parca("İki", ";b", "let x = 1;")).expect("ekleme");
        depo
    }

    #[test]
    fn json_gidis_donusu_korunur() {
        let depo = depo_ornegi();
        let metin = json_uret(&depo).expect("json");
        let geri: Depo = serde_json::from_str(&metin).expect("çöz");
        assert_eq!(depo, geri);
    }

    #[test]
    fn markdown_gidis_donusu_korunur() {
        let depo = depo_ornegi();
        let md = markdown_uret(&depo);
        let geri = markdown_oku(&md).expect("markdown oku");
        assert_eq!(geri.len(), 2);
        assert_eq!(geri[0].ad, "Bir");
        assert_eq!(geri[0].govde, "fn a() {}");
        assert_eq!(geri[0].dil, Dil::Rust);
        assert_eq!(geri[0].etiketler, vec!["et".to_string()]);
        assert_eq!(geri[1].aciklama, "açıklama");
    }

    #[test]
    fn markdown_govde_ici_cit_gidis_donusu_korur() {
        // Gövdenin kendi içinde bir Markdown çiti var: dışa aktarım daha uzun
        // bir çit seçmeli, içe aktarım da bunu gövde saymalı.
        let mut depo = Depo::bos();
        let zorlu = "fn a() { let s = \"```\"; }\n";
        depo.ekle(parca("Citli", ";cit", zorlu)).expect("ekleme");
        let md = markdown_uret(&depo);
        assert!(md.contains("````"), "dışa aktarım daha uzun çit kullanmalı");
        let geri = markdown_oku(&md).expect("markdown oku");
        assert_eq!(geri.len(), 1);
        assert_eq!(geri[0].govde, zorlu.trim_end_matches('\n'));
    }

    #[test]
    fn markdown_ici_uc_tirnakli_govde_korunur() {
        // Üçlü tırnaklı Python doküstrumu: çit yok, yine de korunmalı.
        let mut depo = Depo::bos();
        let govde = "s = '''\nabc\n'''\n";
        depo.ekle(parca("Dokustrum", ";d", govde)).expect("ekleme");
        let geri = markdown_oku(&markdown_uret(&depo)).expect("markdown oku");
        assert_eq!(geri[0].govde, govde.trim_end_matches('\n'));
    }

    #[test]
    fn html_tek_dosya_ve_kacissiz() {
        let depo = depo_ornegi();
        let html = html_uret(&depo);
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<style>"));
        assert!(!html.contains("<link"), "dış kaynak referansı olmamalı");
        assert!(
            !html.contains("src=\"http"),
            "dış kaynak referansı olmamalı"
        );
        assert!(html.contains("Bir"));
    }

    #[test]
    fn html_kacisi_etiketleri_yutar() {
        assert_eq!(kacis("<script>"), "&lt;script&gt;");
        assert_eq!(kacis("a & b"), "a &amp; b");
        assert_eq!(kacis("\"'\""), "&quot;&#39;&quot;");
    }

    #[test]
    fn desteklenmeyen_bicim_hata_verir() {
        assert!(matches!(
            Bicim::ayikla("yaml"),
            Err(Hata::DesteklenmeyenBicim { .. })
        ));
        assert!(matches!(
            Cakisma::ayikla("coz"),
            Err(Hata::DesteklenmeyenBicim { .. })
        ));
    }

    #[test]
    fn ice_aktar_cakisma_politikalari() {
        let metin = json_uret(&depo_ornegi()).expect("json");

        let mut d = depo_ornegi();
        let o = ice_aktar(&mut d, &metin, Bicim::Json, Cakisma::Hata)
            .expect_err("hata politikası durmalı");
        assert!(matches!(o, Hata::YinelenenAd { .. }));

        let mut d = depo_ornegi();
        let o = ice_aktar(&mut d, &metin, Bicim::Json, Cakisma::Atla).expect("atla");
        assert_eq!(o.atlanan, 2);
        assert_eq!(o.eklenen, 0);
        assert!(o.degisiklik_yok());

        let mut d = depo_ornegi();
        let o = ice_aktar(&mut d, &metin, Bicim::Json, Cakisma::Degistir).expect("degistir");
        assert_eq!(o.degisen, 2);
        assert_eq!(d.parcalar.len(), 2, "değiştirme yeni kayıt eklemez");

        let mut d = depo_ornegi();
        let o = ice_aktar(&mut d, &metin, Bicim::Json, Cakisma::YenidenAdlandir)
            .expect("yeniden adlandır");
        assert_eq!(o.yeniden_adlandirilan, 2);
        assert_eq!(d.parcalar.len(), 4);
        assert!(d.adla("Bir-2").is_some());
        assert!(d.adla("İki-2").is_some());
    }

    #[test]
    fn ice_aktar_yeniden_adlandirma_sayaci_artar() {
        // İkinci turda aynı iki ad tekrar gelir; son ek birer birer artar.
        let mut d = depo_ornegi();
        let metin = json_uret(&depo_ornegi()).expect("json");
        ice_aktar(&mut d, &metin, Bicim::Json, Cakisma::YenidenAdlandir).expect("1. tur");
        ice_aktar(&mut d, &metin, Bicim::Json, Cakisma::YenidenAdlandir).expect("2. tur");
        assert!(d.adla("Bir-2").is_some());
        assert!(d.adla("Bir-3").is_some());
        assert!(d.adla("İki-3").is_some());
        assert_eq!(d.parcalar.len(), 6);
    }

    #[test]
    fn ice_aktar_yeni_parca_ekler() {
        let mut d = depo_ornegi();
        let mut yeni = Depo::bos();
        yeni.ekle(parca("Üç", ";c", "fn c() {}")).expect("ekleme");
        let metin = json_uret(&yeni).expect("json");
        let o = ice_aktar(&mut d, &metin, Bicim::Json, Cakisma::Hata).expect("ekle");
        assert_eq!(o.eklenen, 1);
        assert_eq!(d.parcalar.len(), 3);
    }

    #[test]
    fn ice_aktar_markdown_ile_olur() {
        let mut d = Depo::bos();
        let md = markdown_uret(&depo_ornegi());
        let o = ice_aktar(&mut d, &md, Bicim::Markdown, Cakisma::Hata).expect("md");
        assert_eq!(o.eklenen, 2);
        assert_eq!(d.parcalar[0].govde, "fn a() {}");
    }

    #[test]
    fn ice_aktar_html_reddedilir() {
        let mut d = Depo::bos();
        assert!(ice_aktar(&mut d, "<html></html>", Bicim::Html, Cakisma::Atla).is_err());
    }

    #[test]
    fn bozuk_json_ice_aktarim_hata_verir() {
        let mut d = Depo::bos();
        assert!(matches!(
            ice_aktar(&mut d, "{bozuk", Bicim::Json, Cakisma::Atla),
            Err(Hata::Json { .. })
        ));
    }

    #[test]
    fn dogrulama_durumu_depo_semasinda_yazilir() {
        let mut depo = depo_ornegi();
        depo.parcalar[0].dogrulama = Dogrulama::Hatali;
        let metin = json_uret(&depo).expect("json");
        let geri: Depo = serde_json::from_str(&metin).expect("çöz");
        assert_eq!(geri.parcalar[0].dogrulama, Dogrulama::Hatali);
    }
}
