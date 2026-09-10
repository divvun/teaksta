//! The upload gates that stand before the analyser. The language gate itself
//! needs the real models and is exercised in `tests/pipeline_models.rs`.

use super::*;

const PAGE: &str =
    "<html><head><title>t</title></head><body><p>Mun oidnen viesu.</p></body></html>";

fn upload(content: &str, file_name: Option<&str>) -> Upload {
    Upload {
        file_name: file_name.map(str::to_string),
        content: content.as_bytes().to_vec(),
        keep: false,
    }
}

fn rejection(error: anyhow::Error) -> Rejection {
    *error
        .downcast_ref::<Rejection>()
        .unwrap_or_else(|| panic!("a gate rejection, not {error:#}"))
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+2/test]
#[test]
fn a_body_without_a_file_part_is_refused() {
    let directory = tempfile::tempdir().expect("temp dir");

    let missing = store(&upload(PAGE, None), directory.path()).unwrap_err();
    assert_eq!(rejection(missing), Rejection::NoFile);

    let empty = store(&upload("", Some("text.html")), directory.path()).unwrap_err();
    assert_eq!(rejection(empty), Rejection::NoFile);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+2/test]
#[test]
fn the_cap_is_five_megabytes() {
    let directory = tempfile::tempdir().expect("temp dir");
    let padding = " ".repeat(MAX_UPLOAD_BYTES);
    let oversized = format!("{PAGE}{padding}");

    let error = store(&upload(&oversized, Some("text.html")), directory.path()).unwrap_err();

    assert_eq!(MAX_UPLOAD_BYTES, 5 * 1024 * 1024);
    assert_eq!(rejection(error), Rejection::TooLarge);
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+2/test]
#[test]
fn a_non_page_upload_is_refused() {
    let directory = tempfile::tempdir().expect("temp dir");

    let error = store(
        &upload("lemma,tag\nviessu,N\n", Some("t.csv")),
        directory.path(),
    )
    .unwrap_err();

    assert_eq!(rejection(error), Rejection::NotAPage);
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn+1/test]
#[test]
fn markup_or_the_name_makes_a_page() {
    assert!(is_page(PAGE.as_bytes(), "anything"));
    assert!(is_page(b"<!DOCTYPE html><p>x</p>", "anything"));
    assert!(is_page(
        b"<?xml version=\"1.0\"?><x xmlns=\"http://www.w3.org/1999/xhtml\"/>",
        "t"
    ));
    assert!(is_page(b"just words", "lesson.html"));

    assert!(!is_page(b"just words", "lesson.txt"));
    assert!(!is_page(b"", "lesson.html"));
    assert!(!is_page(&[0x00, 0xff, 0xfe, 0x01], "lesson.bin"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.language-gate-fn/test]
#[test]
fn a_cohort_is_known_by_its_readings() {
    let stream = concat!(
        "\"<Mun>\"\n",
        "\t\"mun\" Pron Pers Sg1 Nom\n",
        "\"<qwertz>\"\n",
        "\t\"qwertz\" ?\n",
        "\"<.>\"\n",
        "\t\".\" CLB\n",
    );

    assert_eq!(count_recognised(stream), (2, 1));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.language-gate-fn/test]
#[test]
fn one_known_reading_is_enough() {
    let stream = concat!(
        "\"<Viesut>\"\n",
        "\t\"viesut\" ?\n",
        "\t\"viessu\" N Pl Nom\n",
        "\"<xyzzy>\"\n",
        "\t\"xyzzy\" ?\n",
    );

    assert_eq!(count_recognised(stream), (2, 1));
    assert_eq!(count_recognised(""), (0, 0));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.language-gate-fn/test]
#[test]
fn a_page_without_prose_reads_as_nothing() {
    assert_eq!(
        sme_share("<html><body><script>var a = 1;</script></body></html>").unwrap(),
        0.0
    );
    assert_eq!(SME_READING_SHARE, 0.6);
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+2/test]
#[test]
fn a_stored_upload_is_addressed_by_file_url() {
    let directory = tempfile::tempdir().expect("temp dir");
    let stored = directory.path().join("abc");
    std::fs::write(&stored, PAGE).expect("write");

    let url = file_url(&stored).expect("a file url");

    assert!(url.starts_with("file:///"), "{url}");
    assert!(url.ends_with("/abc"), "{url}");
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn+2/test]
#[test]
fn an_upload_directory_with_a_space_still_addresses() {
    let root = tempfile::tempdir().expect("temp dir");
    // Both hazards a deployment path can carry: a space, which would end the
    // address early, and a non-ASCII character, which has no place in one.
    let directory = root.path().join("upload sadji").join("s\u{e1}mi");
    std::fs::create_dir_all(&directory).expect("the upload directory");
    let stored = directory.join("abc");
    std::fs::write(&stored, PAGE).expect("write");

    let url = file_url(&stored).expect("a file url");

    assert!(!url.contains(' '), "{url}");
    assert!(url.contains("upload%20sadji"), "{url}");
    // The address reads back to the file it names, which is what the
    // enhancement endpoints do with it.
    let parsed = Url::parse(&url).expect("the address parses");
    assert_eq!(parsed.to_file_path().expect("a path"), stored);
}
