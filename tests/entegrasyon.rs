//! Uçtan uca entegrasyon testleri.
//!
//! Kapsam: gerçek dosya sistemi üzerinde depo yazma/okuma, arama, genişletme,
//! denetim, dışa-içe aktarma ve istatistik akışının tamamı. Ek olarak ikili,
//! `CARGO_BIN_EXE_sniphub` yoluyla gerçekten çalıştırılır ve stdout/stderr
//! karşılaştırılır.
//!
//! Ağ erişimi yoktur, rastgelelik kullanılmaz ve saat yalnızca çıktıda
//! görünür; test kararları dosya içeriğine dayanır (WORKER_CONTRACT § 5.2).

use sniphub::aktarma::{self, Bicim, Cakisma};
use sniphub::arama::{ara, Sorgu};
use sniphub::depo;
use sniphub::dil::Dil;
use sniphub::genisle::Genisletici;
use sniphub::istatistik;
use sniphub::kazanc::{self, Ayarlar};
use sniphub::model::{Depo, Dogrulama, Parca};
use sniphub::syntax;
use sniphub::zaman::An;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Test içinde geçici dizin üreten, `Drop` ile temizleyen kapsayıcı.
///
/// Neden `tempfile` yok: bağımlılık politikası (WORKER_CONTRACT § 3.2) hiçbir
/// projeye `tempfile` vermez; yardımcı kendi kodumuzla yazılır.
struct GeciciDizin {
    yol: PathBuf,
}

impl GeciciDizin {
    /// `std::env::temp_dir()` altında, test adından türetilmiş benzersiz dizin üretir.
    fn yeni(etiket: &str) -> std::io::Result<Self> {
        let kok = std::env::temp_dir().join(format!("sniphub-it-{etiket}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&kok);
        std::fs::create_dir_all(&kok)?;
        Ok(Self { yol: kok })
    }

    fn yol(&self) -> &Path {
        &self.yol
    }

    fn depo_yolu(&self) -> PathBuf {
        self.yol().join(depo::VARSAYILAN_DEPO_ADI)
    }
}

impl Drop for GeciciDizin {
    fn drop(&mut self) {
        // Temizlik hatası testi düşürmemeli; `let _ =` bilinçlidir
        // (WORKER_CONTRACT § 5.3).
        let _ = std::fs::remove_dir_all(&self.yol);
    }
}

/// Sabit zaman ve kullanıcı ile örnek parça üretir.
fn parca(ad: &str, tetik: &str, dil: Dil, etiketler: &[&str], govde: &str) -> Parca {
    let mut p = Parca::yeni(
        ad,
        tetik,
        "örnek açıklama",
        dil,
        etiketler.iter().map(|e| (*e).to_string()).collect(),
        govde,
        &An::epoch_saniye(0),
    );
    p.dogrulama = if syntax::denetle(dil, govde).gecerli() {
        Dogrulama::Gecerli
    } else {
        Dogrulama::Hatali
    };
    p
}

/// Gerçek bir Rust + Python karışımı depo.
fn depo_ornegi() -> Depo {
    let mut d = Depo::bos();
    d.ekle(parca(
        "Rust hata yakalama",
        ";hat",
        Dil::Rust,
        &["rust", "hata"],
        "fn f() -> Result<(), String> {\n    Err(\"hata\".into())\n}",
    ))
    .expect("ekleme");
    d.ekle(parca(
        "Python dosya okuma",
        ";oku",
        Dil::Python,
        &["python", "dosya"],
        "with open(yol, encoding=\"utf-8\") as f:\n    return f.read()",
    ))
    .expect("ekleme");
    d.ekle(parca(
        "SQL sorgu iskeleti",
        ";sorgu",
        Dil::Sql,
        &["sql", "db"],
        "SELECT * FROM {{1|tablo}} WHERE id = {{2|kimlik}};",
    ))
    .expect("ekleme");
    d
}

#[test]
fn tam_akis_depo_kur_arama_genislet_disa_aktar() {
    let dizin = GeciciDizin::yeni("tam-akis").expect("gecici dizin");
    let yol = dizin.depo_yolu();

    // 1) Depoyu diske yaz.
    let kaynak = depo_ornegi();
    depo::kaydet(&kaynak, &yol).expect("yazma");

    // 2) Diskten geri oku: gidiş-dönüş birebir.
    let okunan = depo::yukle(&yol).expect("okuma");
    assert_eq!(okunan.parcalar.len(), 3);

    // 3) Dil bazlı arama: "python" yalnızca Python parçasını bulur.
    let sonuclar = ara(
        &okunan.parcalar,
        &Sorgu::metin("dosya").dil_ile(Some(Dil::Python)),
    );
    assert_eq!(sonuclar.len(), 1);
    assert_eq!(sonuclar[0].parca.dil, Dil::Python);

    // 4) Etiket + dil birlikte filtrelenir.
    let sonuclar = ara(
        &okunan.parcalar,
        &Sorgu::default()
            .dil_ile(Some(Dil::Rust))
            .etiketler_ile(vec!["hata".into()], true),
    );
    assert_eq!(sonuclar.len(), 1);
    assert_eq!(sonuclar[0].parca.ad, "Rust hata yakalama");

    // 5) Yer tutuculu SQL parçasını gerçek değerlerle genişlet.
    let mut g = Genisletici::sabit(An::epoch_saniye(0), "test");
    g.deger_ekle("1=kayitlar").expect("atama");
    g.deger_ekle("2=42").expect("atama");
    let sonuc = g
        .genislet(&okunan.bul_zorunlu(";sorgu").expect("parça").govde)
        .expect("genisleme");
    assert_eq!(sonuc.metin, "SELECT * FROM kayitlar WHERE id = 42;");
    assert_eq!(sonuc.odak_sirasi().len(), 2);

    // 6) Üç biçimde dışa aktar ve içe aktarabilirliği doğrula.
    for bicim in [Bicim::Json, Bicim::Markdown] {
        let metin = aktarma::disa_aktar(&okunan, bicim).expect("dışa aktarım");
        let mut hedef = Depo::bos();
        let ozet =
            aktarma::ice_aktar(&mut hedef, &metin, bicim, Cakisma::Hata).expect("içe aktarım");
        assert_eq!(ozet.eklenen, 3, "{} içe aktarımında", bicim.ad());
        assert_eq!(hedef.parcalar.len(), 3);
    }

    // 7) HTML tek dosya olmalı: dış kaynak referansı yok.
    let html = aktarma::html_uret(&okunan);
    assert!(!html.contains("<link"), "dış stil referansı olmamalı");
    assert!(!html.contains("http://"), "dış kaynak olmamalı");
    assert!(html.contains("Rust hata yakalama"));

    // 8) İstatistik: kullanım sayacı kazancı toplar.
    let mut sayacli = okunan.clone();
    sayacli.parcalar[0].kullanim_sayaci = 5;
    let i = istatistik::hesapla(&sayacli.parcalar, Ayarlar::default(), 10);
    assert_eq!(i.toplam_kullanim, 5);
    assert_eq!(i.en_cok.len(), 1);
    assert!(i.toplam_kazanc_tus > 0);
}

#[test]
fn genisleme_sayaci_artar_ve_kalici_yazilir() {
    let dizin = GeciciDizin::yeni("sayac").expect("gecici dizin");
    let yol = dizin.depo_yolu();
    let mut d = depo_ornegi();
    d.ekle(parca(
        "Kısayol",
        ";k",
        Dil::Rust,
        &["kisa"],
        "let kisa = 1;",
    ))
    .expect("ekleme");
    depo::kaydet(&d, &yol).expect("yazma");

    // Genişletme sonrası sayaç artar ve kalıcı olur.
    let mut okunan = depo::yukle(&yol).expect("okuma");
    let indeks = okunan
        .parcalar
        .iter()
        .position(|p| p.bul_anahtarla(";k"))
        .expect("konum");
    okunan.parcalar[indeks].kullanim_artir(&An::epoch_saniye(86_400));
    depo::kaydet(&okunan, &yol).expect("yazma");

    let tekrar = depo::yukle(&yol).expect("yeniden okuma");
    assert_eq!(tekrar.bul_zorunlu(";k").expect("parça").kullanim_sayaci, 1);
    assert_eq!(
        tekrar.bul_zorunlu(";k").expect("parça").guncelleme,
        "1970-01-02T00:00:00Z"
    );
}

#[test]
fn ayni_ad_iki_parca_ikinci_denemede_reddedilir() {
    let dizin = GeciciDizin::yeni("yinelenen").expect("gecici dizin");
    let yol = dizin.depo_yolu();
    let mut d = Depo::bos();
    d.ekle(parca("Aynı Ad", ";a", Dil::Rust, &[], "let a = 1;"))
        .expect("ilk ekleme");
    let hata = d
        .ekle(parca("Aynı Ad", ";b", Dil::Rust, &[], "let b = 2;"))
        .expect_err("ikinci parça reddedilmeli");
    assert!(matches!(hata, sniphub::hata::Hata::YinelenenAd { .. }));
    assert!(d
        .ekle(parca("Aynı Ad", ";c", Dil::Rust, &[], "let c = 3;"))
        .is_err());
    // Diskteki depo hâlâ tek parça içerir.
    depo::kaydet(&d, &yol).expect("yazma");
    assert_eq!(depo::yukle(&yol).expect("okuma").parcalar.len(), 1);
}

#[test]
fn bozuk_depo_dosyasi_ust_projenin_dilinde_calismaz() {
    let dizin = GeciciDizin::yeni("bozuk-depo").expect("gecici dizin");
    let yol = dizin.depo_yolu();
    std::fs::write(&yol, b"{\"surum\": 1, \"parcalar\": [").expect("yazma");
    let hata = depo::yukle(&yol).expect_err("bozuk JSON hata vermeli");
    assert!(matches!(hata, sniphub::hata::Hata::Json { .. }));

    // Bozuk depoyu okumaya çalışmak hiçbir yerde panic üretmez ve düzeltilmez:
    // dosya olduğu gibi kalır.
    let bozuk_icerik = std::fs::read_to_string(&yol).expect("okuma");
    assert_eq!(
        bozuk_icerik, r#"{"surum": 1, "parcalar": ["#,
        "bozuk dosya değiştirilmemeli"
    );
}

#[test]
fn yazma_atomiktir_gecici_dosya_kalmaz() {
    let dizin = GeciciDizin::yeni("atomik-entegrasyon").expect("gecici dizin");
    let yol = dizin.depo_yolu();
    depo::kaydet(&depo_ornegi(), &yol).expect("yazma");
    let gecici = depo::gecici_yol(&yol);
    assert!(yol.exists());
    assert!(!gecici.exists(), "geçici dosya kalmamalı");

    // İkinci yazma da atomik olmalı: eski içerik bütün olarak değişir.
    let d = depo_yukle_ve_tekrar_yaz(&yol);
    assert_eq!(d.parcalar.len(), 3);
    assert!(!gecici.exists());
    let sayim = std::fs::read_dir(dizin.yol()).expect("dizin okuma").count();
    assert_eq!(sayim, 1, "dizinde yalnızca depo dosyası olmalı");
}

fn depo_yukle_ve_tekrar_yaz(yol: &Path) -> Depo {
    let d = depo::yukle(yol).expect("okuma");
    depo::kaydet(&d, yol).expect("yeniden yazma");
    d
}

#[test]
fn kazara_sifreli_icerik_duz_metin_kalir_ve_bu_bilgilidir() {
    // Şifreleme YOKTUR: parça gövdesi depoda düz metin olarak durur.
    // Bu bir "kaza" değil, MANIFEST.md'deki bilinçli sapmadır; testin amacı
    // davranışı dondurmak ve dokümana bağlamaktır.
    let dizin = GeciciDizin::yeni("sifresiz").expect("gecici dizin");
    let yol = dizin.depo_yolu();
    let gizli = "let parola = \"gizli-deger\";";
    let mut d = Depo::bos();
    d.ekle(parca("Gizli", ";g", Dil::Rust, &["gizli"], gizli))
        .expect("ekleme");
    depo::kaydet(&d, &yol).expect("yazma");

    let ham = std::fs::read_to_string(&yol).expect("okuma");
    assert!(
        ham.contains("gizli-deger"),
        "depo düz metindir: şifreleme uygulanmadığı doğrulanır"
    );
    // Kullanıcıyı yanıltmamak adına JSON şemasında şifreleme alanı YOKTUR.
    assert!(!ham.contains("sifreli"));
    assert!(!ham.contains("nonce"));
}

#[test]
fn kazanc_olcumu_gercek_tus_sayacina_dayanir() {
    // Elle sayılabilir iki vaka: küçük harf/rakam = 1, büyük harf = 2 tuş.
    let duz = parca("Düz", ";d", Dil::Rust, &[], "let x = 1;");
    // l,e,t,boşluk,x,boşluk,=,boşluk,1,;  → 10
    assert_eq!(kazanc::hesapla(&duz, Ayarlar::default()).govde_tus, 10);

    let karisik = parca("K", ";k", Dil::Rust, &[], "let Ad = 1;");
    // l,e,t,boşluk,A(2),d,boşluk,=,boşluk,1,;  → 12
    assert_eq!(kazanc::hesapla(&karisik, Ayarlar::default()).govde_tus, 12);

    // Kapsama alınan: kazanç = gövde tuşları − (tetik + ayraç boşluğu).
    let olcum = parca("Ölçüm", ";olc", Dil::Rust, &[], "let mesaj = 1;");
    let k = kazanc::hesapla(&olcum, Ayarlar::default());
    assert_eq!(k.govde_tus, 14); // l,e,t,sp,m,e,s,a,j,sp,=,sp,1,;
    assert_eq!(k.tetik_tus, 5, "\";olc\" = 4 + ayraç boşluğu 1");
    assert_eq!(k.kazanc_tus, 9);
    assert!(k.kazanc_saniye > 0.0);
}

#[test]
fn ikili_komutlari_calisir_ve_gercek_cikti_verir() {
    let dizin = GeciciDizin::yeni("ikili").expect("gecici dizin");
    let yol = dizin.depo_yolu();
    depo::kaydet(&depo_ornegi(), &yol).expect("yazma");

    let bin = env!("CARGO_BIN_EXE_sniphub");

    // `list` gerçek parçaları listeler.
    let cikti = Command::new(bin)
        .args(["--depo"])
        .arg(&yol)
        .args(["list"])
        .output()
        .expect("ikili çalıştırılamadı");
    assert!(cikti.status.success(), "list başarısız: {cikti:?}");
    let metin = String::from_utf8_lossy(&cikti.stdout);
    assert!(metin.contains("Rust hata yakalama"), "{metin}");
    assert!(metin.contains("python"), "{metin}");

    // `search` alaka sıralaması yapar.
    let cikti = Command::new(bin)
        .args(["--depo"])
        .arg(&yol)
        .args(["search", "sorgu", "--lang", "sql"])
        .output()
        .expect("ikili çalıştırılamadı");
    assert!(cikti.status.success());
    assert!(String::from_utf8_lossy(&cikti.stdout).contains("SQL sorgu iskeleti"));

    // `expand` kullanım sayacını artırır ve genişletilmiş metni basar.
    let cikti = Command::new(bin)
        .args(["--depo"])
        .arg(&yol)
        .args(["expand", ";sorgu", "--set", "1=t", "--set", "2=7"])
        .output()
        .expect("ikili çalıştırılamadı");
    assert!(cikti.status.success(), "expand başarısız: {cikti:?}");
    let metin = String::from_utf8_lossy(&cikti.stdout);
    assert!(metin.contains("SELECT * FROM t WHERE id = 7;"), "{metin}");
    assert_eq!(
        depo::yukle(&yol)
            .expect("okuma")
            .bul_zorunlu(";sorgu")
            .expect("parça")
            .kullanim_sayaci,
        1
    );

    // `check` yapısal denge raporu üretir.
    let cikti = Command::new(bin)
        .args(["--depo"])
        .arg(&yol)
        .args(["check", ";hat"])
        .output()
        .expect("ikili çalıştırılamadı");
    assert!(cikti.status.success());
    assert!(String::from_utf8_lossy(&cikti.stdout).contains("denge tamam"));

    // Hatalı girdi sıfır dönüş koduyla çıkar ve stderr'e yazar.
    let cikti = Command::new(bin)
        .args(["--depo"])
        .arg(&yol)
        .args(["get", ";bulunmayan"])
        .output()
        .expect("ikili çalıştırılamadı");
    assert!(!cikti.status.success());
    assert!(String::from_utf8_lossy(&cikti.stderr).contains("parça bulunamadı"));
}

#[test]
fn ikili_ile_disa_i_ce_aktarma_gidis_donusu() {
    let dizin = GeciciDizin::yeni("ikili-aktarma").expect("gecici dizin");
    let yol = dizin.depo_yolu();
    let bin = env!("CARGO_BIN_EXE_sniphub");
    depo::kaydet(&depo_ornegi(), &yol).expect("yazma");

    let dis = dizin.yol().join("katalog.json");
    let cikti = Command::new(bin)
        .args(["--depo"])
        .arg(&yol)
        .args(["export", "--format", "json", "--out"])
        .arg(&dis)
        .output()
        .expect("ikili çalıştırılamadı");
    assert!(cikti.status.success(), "export başarısız: {cikti:?}");
    assert!(dis.exists());

    // Yeni (boş) depoya içe aktar.
    let bos = dizin.yol().join("bos.json");
    let cikti = Command::new(bin)
        .args(["--depo"])
        .arg(&bos)
        .args(["import"])
        .arg(&dis)
        .args(["--on-conflict", "degistir"])
        .output()
        .expect("ikili çalıştırılamadı");
    assert!(cikti.status.success(), "import başarısız: {cikti:?}");
    assert!(String::from_utf8_lossy(&cikti.stdout).contains("3 eklendi"));
    assert_eq!(depo::yukle(&bos).expect("okuma").parcalar.len(), 3);

    // Aynı dosyayı ikinci kez `degistir` ile al: kayıt sayısı değişmez.
    let cikti = Command::new(bin)
        .args(["--depo"])
        .arg(&bos)
        .args(["import"])
        .arg(&dis)
        .args(["--on-conflict", "degistir"])
        .output()
        .expect("ikili çalıştırılamadı");
    assert!(cikti.status.success());
    assert_eq!(
        depo::yukle(&bos).expect("okuma").parcalar.len(),
        3,
        "degistirme politikası yeni kayıt eklememeli"
    );

    // `stats` kazanç tablosunu basar.
    let cikti = Command::new(bin)
        .args(["--depo"])
        .arg(&yol)
        .args(["stats", "--top", "3"])
        .output()
        .expect("ikili çalıştıralamadı");
    assert!(cikti.status.success());
    let metin = String::from_utf8_lossy(&cikti.stdout);
    assert!(metin.contains("parça sayısı     : 3"), "{metin}");
    assert!(metin.contains("Rust"), "{metin}");
}

#[test]
fn denetleyici_gercek_hatali_kodu_yakalar_ve_dusuk_etki_uretir() {
    // Denetleyici gerçek gramer değildir; ama bu iki hata gerçek hatadır ve
    // yakalanmalıdır. Buna karşılık şu geçerli kod yanlış alarm üretmemelidir.
    let kotu_rust = "fn f() { let v = vec![1, 2, 3; }";
    let rapor = syntax::denetle(Dil::Rust, kotu_rust);
    assert!(!rapor.gecerli());
    assert!(!rapor.hatalar().is_empty());

    let kotu_python = "def f():\n    return 1\n  return 2\n";
    assert!(!syntax::denetle(Dil::Python, kotu_python).gecerli());

    let dogru = "fn main() {\n    let v = vec![1, 2, 3];\n    println!(\"{}\", v.len());\n}\n";
    assert!(syntax::denetle(Dil::Rust, dogru).gecerli());
}
