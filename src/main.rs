//! SnipHub komut satırı arayüzü.
//!
//! Kapsam: `clap` ile tanımlanan alt komutların `stdin`/`stdout` üzerinden
//! bağlanması. Tüm iş mantığı `sniphub` kütüphanesindedir; burada yalnızca
//! ayrıştırma, dosya G/Ç ve çıktı biçimlendirmesi vardır.
//!
//! Pano dinleme veya global kısayol kaydı **yoktur**; bu, "programlama
//! parçası" kapsamıyla ve WORKER_CONTRACT.md § 3.2-G ile tutarlıdır.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use clap::{Parser, Subcommand, ValueEnum};
use sniphub::aktarma::{self, Bicim, Cakisma};
use sniphub::arama::{ara, Sorgu};
use sniphub::depo;
use sniphub::dil::Dil;
use sniphub::genisle::Genisletici;
use sniphub::hata::{Hata, Sonuc};
use sniphub::istatistik;
use sniphub::kazanc::{self, Ayarlar};
use sniphub::model::{Depo, Parca};
use sniphub::syntax;
use sniphub::zaman::An;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match calistir(&cli) {
        Ok(cikti) => {
            print!("{cikti}");
            ExitCode::SUCCESS
        }
        Err(hata) => {
            eprintln!("hata: {hata}");
            ExitCode::FAILURE
        }
    }
}

/// SnipHub — programlama parçaları yöneticisi.
#[derive(Parser, Debug)]
#[command(
    name = "sniphub",
    version,
    about = "Programlama parçaları yöneticisi: tetik→kod genişletme, yapısal sözdizimi denetleyicisi, düz metin parça deposu.",
    long_about = "SnipHub, kod parçalarını tetik ile genişletir, Rust ve Python için yapısal denge kontrolü yapar ve dil bazlı arama sunar.\n\nDepo DÜZ METİNDİR: şifreleme yoktur. Parola veya anahtar saklamayın."
)]
struct Cli {
    /// Depo dosyasının yolu (varsayılan: yanındaki `parcalar.json`).
    #[arg(long, global = true, value_name = "YOL")]
    depo: Option<PathBuf>,

    /// Alt komut.
    #[command(subcommand)]
    komut: Komut,
}

/// Dil seçimi için `clap` değer türü.
#[derive(ValueEnum, Clone, Copy, Debug)]
enum DilArg {
    /// Rust
    Rust,
    /// Python
    Python,
    /// JavaScript / TypeScript
    Javascript,
    /// SQL
    Sql,
    /// JSON
    Json,
    /// TOML
    Toml,
    /// Markdown
    Markdown,
    /// Düz metin (25 TypeFast'ın alanı; burada yalnızca etiket olarak bulunur)
    Metin,
}

impl From<DilArg> for Dil {
    fn from(d: DilArg) -> Self {
        match d {
            DilArg::Rust => Dil::Rust,
            DilArg::Python => Dil::Python,
            DilArg::Javascript => Dil::JavaScript,
            DilArg::Sql => Dil::Sql,
            DilArg::Json => Dil::Json,
            DilArg::Toml => Dil::Toml,
            DilArg::Markdown => Dil::Markdown,
            DilArg::Metin => Dil::Metin,
        }
    }
}

/// Dışa/içe aktarma biçimi için `clap` değer türü.
#[derive(ValueEnum, Clone, Copy, Debug)]
enum BicimArg {
    /// JSON
    Json,
    /// Markdown
    Markdown,
    /// HTML
    Html,
}

impl From<BicimArg> for Bicim {
    fn from(b: BicimArg) -> Self {
        match b {
            BicimArg::Json => Bicim::Json,
            BicimArg::Markdown => Bicim::Markdown,
            BicimArg::Html => Bicim::Html,
        }
    }
}

/// Çakışma politikası için `clap` değer türü.
#[derive(ValueEnum, Clone, Copy, Debug)]
enum CakismaArg {
    /// Dur ve hata ver
    Hata,
    /// Var olanı koru
    Atla,
    /// Var olanın yerine geç
    Degistir,
    /// `-2`, `-3` son ekiniyle ekle
    YenidenAdlandir,
}

impl From<CakismaArg> for Cakisma {
    fn from(c: CakismaArg) -> Self {
        match c {
            CakismaArg::Hata => Cakisma::Hata,
            CakismaArg::Atla => Cakisma::Atla,
            CakismaArg::Degistir => Cakisma::Degistir,
            CakismaArg::YenidenAdlandir => Cakisma::YenidenAdlandir,
        }
    }
}

/// SnipHub alt komutları.
#[derive(Subcommand, Debug)]
enum Komut {
    /// Yeni parça ekle
    Add {
        /// Parçanın benzersiz adı
        #[arg(long)]
        name: String,
        /// Genişletme tetiği
        #[arg(long)]
        trigger: String,
        /// Yazıldığı dil
        #[arg(long = "lang", value_enum)]
        lang: DilArg,
        /// Tek satırlık açıklama
        #[arg(long, default_value = "")]
        desc: String,
        /// Virgülle ayrılmış etiketler
        #[arg(long, value_delimiter = ',', default_value = "")]
        tag: Vec<String>,
        /// Gövde metni
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Gövdenin okunacağı dosya
        #[arg(long)]
        body_file: Option<PathBuf>,
    },
    /// Depodaki parçaları listele
    List {
        /// Yalnızca bu dil
        #[arg(long = "lang", value_enum)]
        lang: Option<DilArg>,
        /// Yalnızca bu etikete sahip olanlar
        #[arg(long = "tag", value_delimiter = ',')]
        tag: Vec<String>,
    },
    /// Tek bir parçayı ayrıntılı göster
    Get {
        /// Parça adı veya tetiği
        anahtar: String,
    },
    /// Çoklu dil arama
    Search {
        /// Aranan metin
        metin: String,
        /// Yalnızca bu dil
        #[arg(long = "lang", value_enum)]
        lang: Option<DilArg>,
        /// Etiket filtresi
        #[arg(long = "tag", value_delimiter = ',')]
        tag: Vec<String>,
        /// Etiketlerin hepsi eşleşmeli
        #[arg(long = "her-etiket")]
        her_etiket: bool,
        /// En fazla bu kadar sonuç
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Bir parçayı genişlet
    Expand {
        /// Parça adı veya tetiği
        anahtar: String,
        /// `anahtar=değer` biçiminde yer tutucu ataması (tekrarlanabilir)
        #[arg(long = "set", value_name = "ANAHTAR=DEGER")]
        set: Vec<String>,
        /// Kullanım sayacını artırma
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        sayac: bool,
    },
    /// Yapısal sözdizimi denetimi
    Check {
        /// Denetlenecek parçanın adı veya tetiği (verilmezse stdin okunur)
        anahtar: Option<String>,
        /// stdin'den okunan kodun dili
        #[arg(long = "lang", value_enum)]
        lang: Option<DilArg>,
    },
    /// Dışa aktar
    Export {
        /// Çıktı biçimi
        #[arg(long, value_enum, default_value_t = BicimArg::Json)]
        format: BicimArg,
        /// Yalnızca bu dil
        #[arg(long = "lang", value_enum)]
        lang: Option<DilArg>,
        /// Yazılacak dosya (verilmezse stdout)
        #[arg(long, value_name = "YOL")]
        out: Option<PathBuf>,
    },
    /// İçe aktar
    Import {
        /// Okunacak dosya
        yol: PathBuf,
        /// Giriş biçimi
        #[arg(long, value_enum, default_value_t = BicimArg::Json)]
        format: BicimArg,
        /// Aynı adda parça varsa ne yapılacak
        #[arg(long = "on-conflict", value_enum, default_value_t = CakismaArg::Hata)]
        on_conflict: CakismaArg,
    },
    /// Kullanım ve kazanç istatistiği
    Stats {
        /// En çok kullanılan kaç parça listelensin
        #[arg(long, default_value_t = 10)]
        top: usize,
        /// Yazma hızı (WPM)
        #[arg(long, default_value_t = kazanc::VARSAYILAN_WPM)]
        wpm: f64,
    },
}

/// Seçilen alt komutu çalıştırır ve stdout metnini döner.
fn calistir(cli: &Cli) -> Sonuc<String> {
    let dizin = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let yol = depo::yol_coz(cli.depo.as_deref(), &dizin);
    match &cli.komut {
        Komut::Add {
            name,
            trigger,
            lang,
            desc,
            tag,
            body,
            body_file,
        } => komut_add(
            &yol,
            name,
            trigger,
            Dil::from(*lang),
            desc,
            tag,
            body.as_deref(),
            body_file.as_deref(),
        ),
        Komut::List { lang, tag } => komut_list(&yol, *lang, tag),
        Komut::Get { anahtar } => komut_get(&yol, anahtar),
        Komut::Search {
            metin,
            lang,
            tag,
            her_etiket,
            limit,
        } => komut_search(&yol, metin, *lang, tag, *her_etiket, *limit),
        Komut::Expand {
            anahtar,
            set,
            sayac,
        } => komut_expand(&yol, anahtar, set, *sayac),
        Komut::Check { anahtar, lang } => komut_check(&yol, anahtar.as_deref(), *lang),
        Komut::Export { format, lang, out } => {
            komut_export(&yol, (*format).into(), *lang, out.as_deref())
        }
        Komut::Import {
            yol: kaynak,
            format,
            on_conflict,
        } => komut_import(&yol, kaynak, (*format).into(), (*on_conflict).into()),
        Komut::Stats { top, wpm } => komut_stats(&yol, *top, *wpm),
    }
}

/// `add` alt komutu.
#[allow(clippy::too_many_arguments)]
fn komut_add(
    yol: &std::path::Path,
    ad: &str,
    tetik: &str,
    dil: Dil,
    aciklama: &str,
    etiketler: &[String],
    govde: Option<&str>,
    govde_dosya: Option<&std::path::Path>,
) -> Sonuc<String> {
    if ad.trim().is_empty() {
        return Err(Hata::BozukDepo {
            alan: "ad".into(),
            neden: "ad boş olamaz".into(),
        });
    }
    if tetik.trim().is_empty() {
        return Err(Hata::BozukDepo {
            alan: "tetik".into(),
            neden: "tetik boş olamaz".into(),
        });
    }
    let govde = match (govde, govde_dosya) {
        (Some(g), _) => g.to_string(),
        (None, Some(d)) => {
            std::fs::read_to_string(d).map_err(|kaynak| Hata::io("gövde okunamadı", d, kaynak))?
        }
        (None, None) => {
            return Err(Hata::BozukDepo {
                alan: "govde".into(),
                neden: "`--body` veya `--body-file` zorunludur".into(),
            })
        }
    };
    if govde.trim().is_empty() {
        return Err(Hata::BozukDepo {
            alan: "govde".into(),
            neden: "gövde boş olamaz".into(),
        });
    }

    let (mut depo, yeni_mi) = depo::yukle_veya_bos(yol)?;
    let etiketler: Vec<String> = etiketler
        .iter()
        .map(|e| e.trim().to_string())
        .filter(|e| !e.is_empty())
        .collect();
    let mut parca = Parca::yeni(ad, tetik, aciklama, dil, etiketler, &govde, &An::simdi());
    // Kaydetmeden önce yapısal denetim: rapor § 03, senaryo S1.
    // Kapsam dışı bir dil "hatali" değil "denenmemiş" olarak işaretlenir;
    // "denenmedi" ile "denendi ve hatalı" ayrımı modelin temelidir.
    let denetim = syntax::denetle(dil, &govde);
    parca.dogrulama = if !denetim.destekleniyor {
        sniphub::model::Dogrulama::Denenmemis
    } else if denetim.gecerli() {
        sniphub::model::Dogrulama::Gecerli
    } else {
        sniphub::model::Dogrulama::Hatali
    };
    depo.ekle(parca)?;
    depo::kaydet(&depo, yol)?;
    let _ = yeni_mi;

    let satir = format!(
        "eklendi: {ad} (tetik `{tetik}`, dil {}, denetim: {}){}",
        dil.gorunen_ad(),
        if denetim.gecerli() {
            "tamam"
        } else {
            "tamam değil"
        },
        if yeni_mi {
            ""
        } else {
            " (mevcut depo güncellendi)"
        }
    );
    Ok(format!("{satir}\n{}", denetim_ozeti(&denetim)))
}

/// Denetim sonucunu okunabilir satırlara çevirir.
fn denetim_ozeti(denetim: &syntax::Denetim) -> String {
    if !denetim.destekleniyor {
        return format!(
            "  not: {} için yapısal denge kuralı yok\n",
            denetim.dil.gorunen_ad()
        );
    }
    if denetim.bulgular.is_empty() {
        return "  yapısal denge: temiz\n".to_string();
    }
    let mut c = String::new();
    for bulgu in &denetim.bulgular {
        c.push_str(&format!("  {bulgu}\n"));
    }
    c
}

/// `list` alt komutu.
fn komut_list(yol: &std::path::Path, lang: Option<DilArg>, tag: &[String]) -> Sonuc<String> {
    let (depo, _) = depo::yukle_veya_bos(yol)?;
    let sorgu = Sorgu::default()
        .dil_ile(lang.map(Dil::from))
        .etiketler_ile(tag.to_vec(), true);
    let sonuclar = ara(&depo.parcalar, &sorgu);
    if sonuclar.is_empty() {
        return Ok("depo boş veya filtreye uyan parça yok\n".to_string());
    }
    let mut c = String::new();
    for s in &sonuclar {
        c.push_str(&format!(
            "{:<26} {:<10} {:<12} {:>4} kullanım  {} ({})\n",
            s.parca.ad,
            s.parca.dil.etiket(),
            s.parca.tetik,
            s.parca.kullanim_sayaci,
            s.parca.dogrulama.etiket(),
            s.parca.etiketler.join(", ")
        ));
    }
    Ok(c)
}

/// `get` alt komutu.
fn komut_get(yol: &std::path::Path, anahtar: &str) -> Sonuc<String> {
    let (depo, _) = depo::yukle_veya_bos(yol)?;
    let parca = depo.bul_zorunlu(anahtar)?;
    let k = kazanc::hesapla(parca, Ayarlar::default());
    let saniye = format!("{:.2}", k.kazanc_saniye);
    Ok(format!(
        "ad        : {}\nid        : {}\ntetik     : {}\ndil       : {}\netiketler : {}\naciklama  : {}\nolusturma : {}\nguncelleme: {}\nkullanim  : {}\nkazanc    : {} tuş ({saniye} saniye)\n\n-----\n{}\n-----\n",
        parca.ad,
        parca.id,
        parca.tetik,
        parca.dil.gorunen_ad(),
        parca.etiketler.join(", "),
        parca.aciklama,
        parca.olusturma,
        parca.guncelleme,
        parca.kullanim_sayaci,
        k.kazanc_tus,
        parca.govde.trim_end_matches('\n')
    ))
}

/// `search` alt komutu.
fn komut_search(
    yol: &std::path::Path,
    metin: &str,
    lang: Option<DilArg>,
    tag: &[String],
    her_etiket: bool,
    limit: usize,
) -> Sonuc<String> {
    let (depo, _) = depo::yukle_veya_bos(yol)?;
    let sorgu = Sorgu::metin(metin)
        .dil_ile(lang.map(Dil::from))
        .etiketler_ile(tag.to_vec(), her_etiket);
    let sonuclar = ara(&depo.parcalar, &sorgu);
    if sonuclar.is_empty() {
        return Ok(format!("`{metin}` için eşleşme yok\n"));
    }
    let mut c = format!("{} sonuç:\n", sonuclar.len());
    for s in sonuclar.iter().take(limit) {
        c.push_str(&format!(
            "  [{}] {:<24} {:<10} puan {:>4}  {}\n",
            s.parca.dil.etiket(),
            s.parca.ad,
            s.parca.tetik,
            s.puan,
            s.gerekce.join(", ")
        ));
    }
    Ok(c)
}

/// `expand` alt komutu.
fn komut_expand(
    yol: &std::path::Path,
    anahtar: &str,
    atamalar: &[String],
    sayac: bool,
) -> Sonuc<String> {
    let (mut depo, _) = depo::yukle_veya_bos(yol)?;
    let indeks = depo
        .parcalar
        .iter()
        .position(|p| p.bul_anahtarla(anahtar))
        .ok_or_else(|| Hata::ParcaYok {
            anahtar: anahtar.to_string(),
        })?;

    let mut genisletici = Genisletici::varsayilan();
    for atama in atamalar {
        genisletici.deger_ekle(atama)?;
    }
    let govde = depo.parcalar[indeks].govde.clone();
    let sonuc = genisletici.genislet(&govde)?;

    if sayac {
        let an = An::simdi();
        depo.parcalar[indeks].kullanim_artir(&an);
        depo::kaydet(&depo, yol)?;
    }

    let mut c = String::new();
    for yuva in &sonuc.yuvalar {
        c.push_str(&format!(
            "yuva #{} `{}` satır {} sütun {} [{}] = {}\n",
            yuva.sira,
            yuva.etiket,
            yuva.satir,
            yuva.sutun,
            if yuva.dolduruldu {
                "doldu"
            } else {
                "varsayılan"
            },
            yuva.deger
        ));
    }
    c.push_str("-----\n");
    c.push_str(sonuc.metin.trim_end_matches('\n'));
    c.push('\n');
    Ok(c)
}

/// `check` alt komutu.
fn komut_check(
    yol: &std::path::Path,
    anahtar: Option<&str>,
    lang: Option<DilArg>,
) -> Sonuc<String> {
    let (kod, dil) = match (anahtar, lang) {
        (Some(a), _) => {
            let (depo, _) = depo::yukle_veya_bos(yol)?;
            let parca = depo.bul_zorunlu(a)?;
            (parca.govde.clone(), parca.dil)
        }
        (None, Some(l)) => {
            let mut s = String::new();
            std::io::stdin()
                .read_to_string(&mut s)
                .map_err(|kaynak| Hata::io("stdin okunamadı", "<stdin>", kaynak))?;
            (s, Dil::from(l))
        }
        (None, None) => {
            return Err(Hata::BilinmeyenDil {
                etiket: "--lang verilmedi".into(),
            })
        }
    };
    let denetim = syntax::denetle(dil, &kod);
    let baslik = format!(
        "{} · {} · {} bulgu\n",
        dil.gorunen_ad(),
        if denetim.gecerli() {
            "denge tamam"
        } else {
            "denge tamam değil"
        },
        denetim.bulgular.len()
    );
    Ok(format!("{baslik}{}", denetim_ozeti(&denetim)))
}

/// `export` alt komutu.
fn komut_export(
    yol: &std::path::Path,
    bicim: Bicim,
    lang: Option<DilArg>,
    out: Option<&std::path::Path>,
) -> Sonuc<String> {
    let (depo, _) = depo::yukle_veya_bos(yol)?;
    let depo = match lang.map(Dil::from) {
        Some(d) => Depo {
            surum: depo.surum,
            parcalar: depo.parcalar.into_iter().filter(|p| p.dil == d).collect(),
        },
        None => depo,
    };
    let metin = aktarma::disa_aktar(&depo, bicim)?;
    match out {
        Some(hedef) => {
            let mut dosya = std::fs::File::create(hedef)
                .map_err(|kaynak| Hata::io("çıktı oluşturulamadı", hedef, kaynak))?;
            dosya
                .write_all(metin.as_bytes())
                .map_err(|kaynak| Hata::io("çıktı yazılamadı", hedef, kaynak))?;
            Ok(format!(
                "{} parça {} biçiminde yazıldı: {} ({} bayt)\n",
                depo.parcalar.len(),
                bicim.ad(),
                hedef.display(),
                metin.len()
            ))
        }
        None => Ok(metin),
    }
}

/// `import` alt komutu.
fn komut_import(
    yol: &std::path::Path,
    kaynak: &std::path::Path,
    bicim: Bicim,
    politika: Cakisma,
) -> Sonuc<String> {
    let metin = std::fs::read_to_string(kaynak)
        .map_err(|hata| Hata::io("içe aktarılacak dosya okunamadı", kaynak, hata))?;
    let (mut depo, _) = depo::yukle_veya_bos(yol)?;
    let ozet = aktarma::ice_aktar(&mut depo, &metin, bicim, politika)?;
    depo::kaydet(&depo, yol)?;
    Ok(format!(
        "içe aktarıldı: {} eklendi, {} değişti, {} yeniden adlandırıldı, {} atlandı (politika: {})\n",
        ozet.eklenen,
        ozet.degisen,
        ozet.yeniden_adlandirilan,
        ozet.atlanan,
        politika.ad()
    ))
}

/// `stats` alt komutu.
fn komut_stats(yol: &std::path::Path, top: usize, wpm: f64) -> Sonuc<String> {
    let (depo, _) = depo::yukle_veya_bos(yol)?;
    let ayarlar = Ayarlar {
        wpm,
        ..Ayarlar::default()
    };
    let i = istatistik::hesapla(&depo.parcalar, ayarlar, top);
    let mut c = format!(
        "parça sayısı     : {}\ntoplam genişletme : {}\nkazanan tuş      : {}\nkazanan süre      : {:.1} saniye ({:.1} dakika)\nhız varsayımı     : {:.0} WPM\nkazancı olmayan    : {} parça (%{:.0})\n\ndil dağılımı:\n",
        i.parca_sayisi,
        i.toplam_kullanim,
        i.toplam_kazanc_tus,
        i.toplam_kazanc_saniye,
        i.toplam_dakika(),
        wpm,
        i.kazanci_olmayan,
        i.ise_yaramaz_orani() * 100.0
    );
    for (dil, adet) in &i.dagilim {
        c.push_str(&format!("  {:<12} {}\n", dil.gorunen_ad(), adet));
    }
    if i.en_cok.is_empty() {
        c.push_str("\nhenüz genişletme kaydı yok\n");
    } else {
        c.push_str(&format!("\nen çok kullanılan {}:\n", i.en_cok.len()));
        for satir in &i.en_cok {
            c.push_str(&format!("  {satir}\n"));
        }
    }
    Ok(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dil_arg_donusumu_tutarli() {
        assert_eq!(Dil::from(DilArg::Rust), Dil::Rust);
        assert_eq!(Dil::from(DilArg::Metin), Dil::Metin);
        assert_eq!(Bicim::from(BicimArg::Html), Bicim::Html);
        assert_eq!(Cakisma::from(CakismaArg::Atla), Cakisma::Atla);
    }

    #[test]
    fn cli_komutlari_yorumlanir() {
        let cli = Cli::try_parse_from([
            "sniphub", "search", "sorgu", "--lang", "rust", "--tag", "db,sql", "--limit", "5",
        ])
        .expect("ayrıştırma");
        match cli.komut {
            Komut::Search {
                metin,
                lang,
                tag,
                her_etiket,
                limit,
            } => {
                assert_eq!(metin, "sorgu");
                assert!(matches!(lang, Some(DilArg::Rust)));
                assert_eq!(tag, vec!["db".to_string(), "sql".to_string()]);
                assert!(!her_etiket);
                assert_eq!(limit, 5);
            }
            _ => panic!("yanlış alt komut"),
        }
    }

    #[test]
    fn add_govde_ve_dosya_birlikte_verilemez() {
        assert!(Cli::try_parse_from([
            "sniphub",
            "add",
            "--name",
            "a",
            "--trigger",
            ";a",
            "--lang",
            "rust",
            "--body",
            "fn a() {}",
            "--body-file",
            "x.rs"
        ])
        .is_err());
    }
}
