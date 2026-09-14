//! The fetch layer's own decisions: which addresses are public, which files
//! this deployment serves, and how a path is resolved before either is
//! judged. Nothing here opens a socket.

use super::*;

use std::path::Path;

/// How much of a page these tests let through: well over every fixture here
/// that is meant to be read, and small enough that a fixture meant not to be
/// is written in one line rather than weighing megabytes.
const TEST_CAP: usize = 64 * 1024;

fn config_under(root: &Path) -> Config {
    Config {
        listen: "127.0.0.1:0".to_string(),
        webapp_dist: None,
        topics: None,
        analysis_dir: root.join("analysed"),
        upload_keep_dir: root.join("keep"),
        upload_temp_dir: root.join("temp"),
        // Nothing here goes through the HTTP surface, so no request is
        // counted and the client a limiter would count is never resolved.
        trust_proxy: false,
        rate_limit: None,
        max_page_bytes: TEST_CAP,
        // Nothing here reaches Azure, and the store these tests read through
        // is the local one rooted at the keep directory.
        azure: None,
    }
}

/// A deployment laid out as a running one is, with one page already stored in
/// each directory a `file:` URL is legitimately minted under.
fn deployment() -> (tempfile::TempDir, Config) {
    let root = tempfile::tempdir().expect("temp dir");
    let config = config_under(root.path());
    for directory in [
        &config.analysis_dir,
        &config.upload_keep_dir,
        &config.upload_temp_dir,
    ] {
        std::fs::create_dir_all(directory).expect("a deployment directory");
    }
    (root, config)
}

fn of(path: &Path) -> String {
    Url::from_file_path(path)
        .expect("an absolute path")
        .to_string()
}

/// The store this deployment keeps texts in, which for every test here is the
/// local one under the keep directory.
fn store(config: &Config) -> TextStore {
    TextStore::from_config(config).expect("the text store opens")
}

#[test]
fn the_local_host_is_never_public() {
    for literal in [
        "127.0.0.1",
        "127.255.255.254",
        "0.0.0.0",
        "0.1.2.3",
        "10.0.0.1",
        "172.16.5.4",
        "172.31.255.255",
        "192.168.1.1",
        // The instance-metadata address every cloud answers on.
        "169.254.169.254",
        "100.64.0.1",
        "192.0.0.1",
        "198.18.0.1",
        "240.0.0.1",
        "255.255.255.255",
        "224.0.0.1",
        "::1",
        "::",
        "fd00::1",
        "fe80::1",
        "ff02::1",
        "2001:db8::1",
        // The IPv4 forms that live inside an IPv6 address.
        "::ffff:127.0.0.1",
        "::ffff:169.254.169.254",
        "::127.0.0.1",
        "2002:7f00:1::",
        "64:ff9b::a00:1",
    ] {
        let ip: IpAddr = literal.parse().expect("an address");
        assert!(!is_public(ip), "{literal} must not count as public");
    }
}

#[test]
fn a_public_address_stays_public() {
    for literal in [
        "1.1.1.1",
        "8.8.8.8",
        "129.242.4.1",
        "172.15.255.255",
        "172.32.0.1",
        "100.63.255.255",
        "100.128.0.1",
        "198.20.0.1",
        "2606:4700::1111",
        "2002:8080:8080::",
        "64:ff9b::808:808",
    ] {
        let ip: IpAddr = literal.parse().expect("an address");
        assert!(is_public(ip), "{literal} must count as public");
    }
}

#[test]
fn a_private_literal_is_refused_before_connecting() {
    let (_root, config) = deployment();

    for raw in [
        "http://127.0.0.1/a",
        "http://127.0.0.1:8080/a",
        "https://169.254.169.254/latest/meta-data/",
        "http://10.1.2.3/a",
        "http://192.168.0.1/a",
        "http://[::1]/a",
        "http://[::ffff:169.254.169.254]/a",
        // The legacy spellings of 127.0.0.1, which the URL parser normalises
        // to a dotted quad before anything here reads the host.
        "http://2130706433/a",
        "http://0177.0.0.1/a",
        "http://0x7f.1/a",
        // Reserved for the local host whatever a resolver would answer.
        "http://localhost/a",
        "http://LocalHost./a",
        "http://api.localhost/a",
    ] {
        assert_eq!(
            target(raw, &config),
            Err(Refusal::Private),
            "{raw} must be refused"
        );
    }
}

#[test]
fn a_public_address_becomes_a_web_target() {
    let (_root, config) = deployment();

    for raw in ["http://example.org/artihkal", "https://8.8.8.8/a"] {
        let target = target(raw, &config).expect("a public address is fetchable");

        assert!(!target.is_stored(), "{raw} must be fetched, not read");
        assert_eq!(target.address(), raw);
    }
}

#[test]
fn only_three_schemes_are_fetchable() {
    let (_root, config) = deployment();

    for raw in [
        "ftp://example.org/a",
        "jar:file:///srv/a.jar!/b.html",
        "gopher://example.org/a",
    ] {
        assert_eq!(
            target(raw, &config),
            Err(Refusal::Scheme),
            "{raw} must be refused"
        );
    }

    // A `data:` URL never gets as far as having a scheme read off it. It
    // carries no authority, so it is not one of the forms an address is
    // recognised as absolute by, and read as a bare host it is not an address
    // at all. Refused either way, and refused before anything is opened.
    assert_eq!(
        target("data:text/html,<p>a</p>", &config),
        Err(Refusal::Address)
    );
}

/// A stored-text reference is read from the store, and nothing else that
/// merely resembles one is.
///
/// The two halves are the whole of why the recognition opens no hole. A
/// reference carries no authority component, so there is nothing in it a
/// caller could have chosen and nothing to compare against a hostname this
/// process cannot know; and an address that *does* carry one is an address of
/// that host, judged by the private-network policy exactly as any other
/// address of it would be, whatever its path spells.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+7/test]
#[tokio::test]
async fn a_stored_text_is_read_from_the_store() {
    let (_root, config) = deployment();
    let store = store(&config);
    let page = "<html><body><p>Mun oidnen viesu.</p></body></html>";
    let id = store
        .put(page.as_bytes().to_vec())
        .await
        .expect("the text is stored");
    let reference = id.reference();

    let stored = target(&reference, &config).expect("a stored text is readable");
    assert!(stored.is_stored(), "{reference} must be read, not fetched");
    assert_eq!(stored.address(), reference);
    assert_eq!(
        fetch(stored, &store)
            .await
            .expect("the stored text is read"),
        page
    );

    // The same name on a host is that host's address, and is refused by the
    // policy that refuses every private address — not admitted by its path.
    for raw in [
        format!("http://127.0.0.1{reference}"),
        format!("http://localhost{reference}"),
        format!("http://169.254.169.254{reference}"),
        format!("http://[::1]{reference}"),
        // The protocol-relative form names a host by leaving out only the
        // scheme, so it is not a reference to anything here.
        format!("/{reference}"),
    ] {
        assert!(
            target(&raw, &config).is_err(),
            "{raw} must not be read from the store"
        );
    }

    // A public host with the same path is an ordinary web address: fetched,
    // not read.
    let elsewhere = format!("http://example.org{reference}");
    let fetched = target(&elsewhere, &config).expect("a public address is fetchable");
    assert!(
        !fetched.is_stored(),
        "{elsewhere} must be fetched from the host it names"
    );
    assert_eq!(fetched.address(), elsewhere);
}

/// What a reference may hold, at the seam that reads one. Nothing a caller
/// writes becomes an object key, so every shape that would mean something to
/// a path is refused here rather than at the store.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.do-get-fn+7/test]
#[test]
fn a_reference_holds_a_name_and_nothing_else() {
    let (_root, config) = deployment();

    for raw in [
        "/api/texts/",
        "/api/texts/nonsense",
        "/api/texts/../../etc/passwd",
        "/api/texts/0123456789abcdef0123456789abcde",
        "/api/texts/0123456789ABCDEF0123456789abcdef",
        "/api/texts/0123456789abcdef0123456789abcdef/more",
    ] {
        assert_eq!(
            target(raw, &config),
            Err(Refusal::Address),
            "{raw} must not name a stored text"
        );
    }

    // A root-relative path that is not a reference at all names nothing this
    // deployment holds and is not turned into a host either.
    for raw in ["/", "/etc/passwd", "/api/enhance", "/api/texts"] {
        assert_eq!(
            target(raw, &config),
            Err(Refusal::Scheme),
            "{raw} must not be reachable"
        );
    }
}

#[test]
fn a_file_this_deployment_serves_is_readable() {
    let (_root, config) = deployment();

    // The two upload directories are the whole list: an accepted upload is
    // the only `file:` URL this deployment ever hands a client.
    for directory in [&config.upload_keep_dir, &config.upload_temp_dir] {
        let page = directory.join("artihkal.html");
        std::fs::write(&page, "<p>a</p>").expect("a page");

        let target = target(&of(&page), &config).expect("a served file is readable");

        assert!(target.is_stored(), "{} must be read", page.display());
    }
}

#[test]
fn a_file_outside_the_served_directories_is_refused() {
    let (root, config) = deployment();
    let elsewhere = root.path().join("elsewhere.html");
    std::fs::write(&elsewhere, "<p>a</p>").expect("a page");

    for path in [
        Path::new("/etc/passwd"),
        elsewhere.as_path(),
        // The analysis cache holds documents, not pages a caller may read.
        &config.analysis_dir.join("v3-0123456789abcdef.json"),
        // A neighbour whose name merely begins the same way.
        &root.path().join("keep-elsewhere").join("a.html"),
    ] {
        assert_eq!(
            target(&of(path), &config),
            Err(Refusal::Confined),
            "{} must be refused",
            path.display()
        );
    }
}

#[test]
fn a_symlink_leaving_a_served_directory_is_refused() {
    let (root, config) = deployment();
    let secret = root.path().join("secret.html");
    std::fs::write(&secret, "<p>secret</p>").expect("the secret");

    let planted = config.upload_temp_dir.join("planted.html");
    std::os::unix::fs::symlink(&secret, &planted).expect("the symlink");
    let through_a_directory = config.upload_temp_dir.join("out").join("secret.html");
    std::os::unix::fs::symlink(root.path(), config.upload_temp_dir.join("out"))
        .expect("the directory symlink");

    for path in [planted.as_path(), through_a_directory.as_path()] {
        assert_eq!(
            target(&of(path), &config),
            Err(Refusal::Confined),
            "{} must not escape",
            path.display()
        );
    }
}

#[test]
fn climbing_out_of_a_served_directory_is_refused() {
    let (root, config) = deployment();
    std::fs::write(root.path().join("secret.html"), "<p>secret</p>").expect("the secret");

    // The URL parser resolves `..` itself, so this arrives already climbed;
    // the confinement is what refuses it.
    let raw = format!(
        "{}/../secret.html",
        of(&config.upload_temp_dir).as_str().trim_end_matches('/')
    );

    assert_eq!(
        target(&raw, &config),
        Err(Refusal::Confined),
        "{raw} must be refused"
    );
}

/// A page with enough chrome on it that the reduction would cut it down —
/// which is what makes reading one off the disk unchanged mean something.
fn a_reducible_page() -> String {
    let mut page = String::from(concat!(
        "<!DOCTYPE html><html><head><title>Beana viehk\u{e1} olgun</title></head><body>",
        "<nav><ul><li><a href=\"/ovdasiidu\">Ovdasiidu</a></li>",
        "<li><a href=\"/searvvus\">Searvvus</a></li>",
        "<li><a href=\"/gulahallan\">Gulahallan</a></li></ul></nav>",
        "<article>"
    ));
    for _ in 0..6 {
        page.push_str(
            "<p>Mun oidnen viesu ikte. Viesut leat stuorr\u{e1}t ja alit, ja sii leat \
             huksejuvvon boarr\u{e1}siid \u{e1}iggis. B\u{e1}rdni lea skuvllas odne, ja \
             nieida logai girjji mii lei beavddis.</p>",
        );
    }
    page.push_str("</article><footer><p>Priv\u{e1}htavuohta</p></footer></body></html>");
    page
}

/// The scope of the reader-mode reduction, at the seam that decides it: a
/// page read off the disk is the one this deployment was given — an accepted
/// upload, or a page shipped with an activity — and is handed on exactly as
/// it was written, chrome and all.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.reader-fn/test]
#[tokio::test]
async fn a_file_is_read_as_it_was_written() {
    let (_root, config) = deployment();
    let page = a_reducible_page();
    // The fixture has to be one the reduction would act on, or the assertion
    // below would hold for the wrong reason.
    assert_ne!(
        reader::reduce(page.clone(), "http://example.org/artihkal"),
        page,
        "this page is not one the reduction would change"
    );
    let at = config.upload_temp_dir.join("artihkal.html");
    std::fs::write(&at, &page).expect("a stored page");

    let target = target(&of(&at), &config).expect("a served file is readable");
    let read = fetch(target, &store(&config))
        .await
        .expect("the stored page is read");

    assert_eq!(read, page, "a page off the disk was reduced");
}

/// The cap bounds the read itself: one byte past it is all that is ever held,
/// and the page is refused rather than cut down, because half a document
/// analysed as a whole one is a worse answer than none.
///
/// A far end is not needed to show this. The fetch layer refuses every
/// private address, so an HTTP server on this machine is one this deployment
/// will not read from, and what the cap actually guards — the read — is the
/// same read whatever produced the bytes.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn+1/test]
#[test]
fn a_read_stops_one_byte_past_the_cap() {
    let at_the_cap = "a".repeat(TEST_CAP);
    let one_more = "a".repeat(TEST_CAP + 1);

    assert_eq!(
        capped(at_the_cap.as_bytes(), TEST_CAP, "http://example.org/a")
            .expect("a page at the cap is read"),
        at_the_cap.as_bytes()
    );

    let Err(error) = capped(one_more.as_bytes(), TEST_CAP, "http://example.org/a") else {
        panic!("a page one byte over the cap must be refused");
    };
    let oversized = error
        .downcast_ref::<Oversized>()
        .expect("an oversized page names itself");
    assert_eq!(oversized.cap, TEST_CAP);
    assert_eq!(oversized.address, "http://example.org/a");

    // A source that never ends is abandoned at the cap rather than read for
    // as long as it streams.
    let endless = std::io::repeat(b'a');
    assert!(
        capped(endless, TEST_CAP, "http://example.org/a")
            .unwrap_err()
            .downcast_ref::<Oversized>()
            .is_some()
    );
}

/// The cap applies to a page off the disk as much as to one off the network.
/// Every `file:` address this deployment reads names something it stored
/// itself, under a limit of its own, so a larger file in a served directory
/// is a directory holding something the deployment did not put there.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn+1/test]
#[tokio::test]
async fn a_stored_page_over_the_cap_is_refused() {
    let (_root, config) = deployment();
    let over = config.upload_temp_dir.join("enormous.html");
    // Non-ASCII, so an oversized page is reported as oversized rather than as
    // invalid UTF-8 at whichever byte the cap happened to fall inside.
    std::fs::write(&over, "á".repeat(TEST_CAP)).expect("an oversized page");
    let under = config.upload_temp_dir.join("ordinary.html");
    std::fs::write(&under, "<p>Mun oidnen viesu.</p>").expect("an ordinary page");

    let enormous = target(&of(&over), &config).expect("a served file is readable");
    let error = fetch(enormous, &store(&config))
        .await
        .expect_err("an oversized page is refused");

    assert!(
        error.downcast_ref::<Oversized>().is_some(),
        "{error:#} is not an oversized page"
    );
    // The address was never the problem, so the ordinary page beside it is
    // still read.
    let ordinary = target(&of(&under), &config).expect("a served file is readable");
    assert_eq!(
        fetch(ordinary, &store(&config))
            .await
            .expect("an ordinary page is read"),
        "<p>Mun oidnen viesu.</p>"
    );
}

/// The charset a response declares is honoured, which is what reading the
/// body ourselves rather than letting the client buffer it has to keep.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn+1/test]
#[test]
fn a_declared_charset_is_read() {
    assert_eq!(
        charset_parameter("text/html; charset=iso-8859-1"),
        Some("iso-8859-1")
    );
    assert_eq!(
        charset_parameter("text/html;charset=\"UTF-8\""),
        Some("UTF-8")
    );
    assert_eq!(
        charset_parameter("text/html; boundary=x; Charset = latin1"),
        Some("latin1")
    );
    assert_eq!(charset_parameter("text/html"), None);
    // The parameter is a parameter: a media type that merely contains the
    // word is not one.
    assert_eq!(charset_parameter("text/charset=1"), None);

    let mut headers = HeaderMap::new();
    assert_eq!(charset(&headers).name(), UTF_8.name());
    headers.insert(
        CONTENT_TYPE,
        "text/html; charset=iso-8859-1".parse().unwrap(),
    );
    assert_eq!(charset(&headers).name(), "windows-1252");
    headers.insert(
        CONTENT_TYPE,
        "text/html; charset=not-an-encoding".parse().unwrap(),
    );
    assert_eq!(charset(&headers).name(), UTF_8.name());
}

#[test]
fn a_file_not_yet_written_is_still_placed() {
    let (_root, config) = deployment();
    let missing = config.upload_temp_dir.join("deep").join("absent.html");

    // A stored upload that has since been swept is inside the deployment and
    // therefore unreadable rather than refused; the read reports that.
    let target = target(&of(&missing), &config).expect("a path inside a served directory");

    assert!(target.is_stored());
}

#[test]
fn a_deployment_serving_nothing_serves_nothing() {
    let root = tempfile::tempdir().expect("temp dir");
    // None of the configured directories exists, so none of them resolves.
    let config = config_under(&root.path().join("absent"));

    assert!(served_roots(&config).is_empty());
    assert_eq!(
        target(&of(Path::new("/etc/passwd")), &config),
        Err(Refusal::Confined)
    );
}

#[test]
fn resolution_follows_what_exists_and_keeps_the_rest() {
    let root = tempfile::tempdir().expect("temp dir");
    let real = root.path().join("real");
    std::fs::create_dir_all(&real).expect("the directory");
    let link = root.path().join("link");
    std::os::unix::fs::symlink(&real, &link).expect("the symlink");

    let resolved = resolve(&link.join("deep").join("page.html")).expect("a resolvable path");

    let real = real.canonicalize().expect("a real directory");
    assert_eq!(resolved, real.join("deep").join("page.html"));
    // A climb above everything that exists cannot be resolved against a real
    // directory, so it is not guessed at either.
    assert_eq!(resolve(Path::new("/teaksta-absent-9f2/deep/..")), None);
    assert_eq!(resolve(Path::new("/")), Some(PathBuf::from("/")));
}
