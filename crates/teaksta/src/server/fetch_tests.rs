//! The fetch layer's own decisions: which addresses are public, which files
//! this deployment serves, and how a path is resolved before either is
//! judged. Nothing here opens a socket.

use super::*;

use std::path::Path;

fn config_under(root: &Path) -> Config {
    Config {
        listen: "127.0.0.1:0".to_string(),
        webapp_root: root.join("webapp"),
        webapp_dist: None,
        classpath_root: root.join("desc"),
        activities_dir: root.join("webapp").join("activities"),
        analysis_dir: root.join("analysed"),
        upload_keep_dir: root.join("keep"),
        upload_temp_dir: root.join("temp"),
    }
}

/// A deployment laid out as a running one is, with one page already stored in
/// each directory a `file:` URL is legitimately minted under.
fn deployment() -> (tempfile::TempDir, Config) {
    let root = tempfile::tempdir().expect("temp dir");
    let config = config_under(root.path());
    for directory in [
        &config.webapp_root,
        &config.activities_dir,
        &config.analysis_dir,
        &config.upload_keep_dir,
        &config.upload_temp_dir,
    ] {
        std::fs::create_dir_all(directory).expect("a deployment directory");
    }
    (root, config)
}

fn address(url: &str) -> Url {
    Url::parse(url).expect("a parsable address")
}

fn of(path: &Path) -> Url {
    Url::from_file_path(path).expect("an absolute path")
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
            target(&address(raw), &config),
            Err(Refusal::Private),
            "{raw} must be refused"
        );
    }
}

#[test]
fn a_public_address_becomes_a_web_target() {
    let (_root, config) = deployment();

    for raw in ["http://example.org/artihkal", "https://8.8.8.8/a"] {
        let target = target(&address(raw), &config).expect("a public address is fetchable");

        assert!(!target.is_file(), "{raw} must be fetched, not read");
        assert_eq!(target.address(), raw);
    }
}

#[test]
fn only_three_schemes_are_fetchable() {
    let (_root, config) = deployment();

    for raw in [
        "ftp://example.org/a",
        "data:text/html,<p>a</p>",
        "jar:file:///srv/a.jar!/b.html",
        "gopher://example.org/a",
    ] {
        assert_eq!(
            target(&address(raw), &config),
            Err(Refusal::Scheme),
            "{raw} must be refused"
        );
    }
}

#[test]
fn a_file_this_deployment_serves_is_readable() {
    let (_root, config) = deployment();

    for directory in [
        &config.upload_keep_dir,
        &config.upload_temp_dir,
        &config.activities_dir,
        &config.webapp_root,
    ] {
        let page = directory.join("artihkal.html");
        std::fs::write(&page, "<p>a</p>").expect("a page");

        let target = target(&of(&page), &config).expect("a served file is readable");

        assert!(target.is_file(), "{} must be read", page.display());
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
        &config.analysis_dir.join("cas_0.xmi"),
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
        target(&address(&raw), &config),
        Err(Refusal::Confined),
        "{raw} must be refused"
    );
}

#[test]
fn a_file_not_yet_written_is_still_placed() {
    let (_root, config) = deployment();
    let missing = config.upload_temp_dir.join("deep").join("absent.html");

    // A stored upload that has since been swept is inside the deployment and
    // therefore unreadable rather than refused; the read reports that.
    let target = target(&of(&missing), &config).expect("a path inside a served directory");

    assert!(target.is_file());
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
