//! The store's own decisions: what a name may be, what a write does to a name
//! that is taken, and what a read refuses. Everything here runs against the
//! local backing, which is the one a deployment without Azure gets and the one
//! every test and every laptop uses.
//!
//! What is not tested here is Azure itself. Both backings are `object_store`'s
//! and the code above this seam never learns which it has, so what an Azure
//! test would exercise is `object_store`'s shared-key signing rather than
//! anything written here. The configuration that chooses between the two is
//! tested where it is read, in `context.rs`.

use super::*;

const PAGE: &str = "<html><body><p>Mun oidnen viesu.</p></body></html>";

fn store() -> (tempfile::TempDir, TextStore) {
    let root = tempfile::tempdir().expect("temp dir");
    let store = TextStore::local(&root.path().join("keep")).expect("the store opens");
    (root, store)
}

/// A deployment naming whichever of the two stores the caller wants it to.
/// Nothing else about it is read here.
fn configured(keep: Option<PathBuf>, azure: bool) -> Config {
    Config {
        listen: "127.0.0.1:0".to_string(),
        webapp_dist: None,
        topics: None,
        analysis_dir: PathBuf::from("analysed"),
        upload_keep_dir: keep,
        upload_temp_dir: PathBuf::from("temp"),
        trust_proxy: false,
        rate_limit: None,
        max_page_bytes: 5 * 1024 * 1024,
        azure: azure.then(|| AzureStorage {
            account: "teakstasa".to_string(),
            container: "kept-texts".to_string(),
            access_key: "c2VjcmV0".to_string(),
        }),
    }
}

/// What a deployment names is what it stores in, and naming nothing is a
/// deployment that stores nothing.
///
/// Both named is Azure: the keep directory is the switch a laptop turns the
/// flow on with, and a pod carrying both is one whose scratch directory was
/// left in the manifest beside the secret somebody has since minted. Building
/// the Azure client opens no socket, so this reaches nothing.
// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn+1/test]
#[test]
fn the_store_is_the_one_the_deployment_named() {
    let root = tempfile::tempdir().expect("temp dir");
    let keep = root.path().join("keep");

    let local = TextStore::from_config(&configured(Some(keep.clone()), false))
        .expect("the store opens")
        .expect("a named directory is a store");
    assert!(!local.is_durable());
    assert!(local.describe().contains("keep"), "{}", local.describe());

    let azure = TextStore::from_config(&configured(None, true))
        .expect("the store opens")
        .expect("a named container is a store");
    assert!(azure.is_durable());
    assert!(
        azure.describe().contains("kept-texts"),
        "{}",
        azure.describe()
    );

    // Both named, and the container wins.
    let both = TextStore::from_config(&configured(Some(keep), true))
        .expect("the store opens")
        .expect("a named container is a store");
    assert!(both.is_durable(), "{}", both.describe());
    assert!(
        both.describe().contains("kept-texts"),
        "{}",
        both.describe()
    );

    // Neither named, and there is no store at all — which is a deployment
    // that takes no uploads rather than one that fails.
    assert!(
        TextStore::from_config(&configured(None, false))
            .expect("no store is not a failure")
            .is_none()
    );
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn+1/test]
#[test]
fn a_name_is_a_digest_and_nothing_else() {
    let id = TextId::of(PAGE.as_bytes());

    assert_eq!(id.as_str().len(), TEXT_ID_LEN);
    assert!(id.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(id.reference(), format!("/api/texts/{id}"));
    // The name is the content's, so the same text is the same name whoever
    // offered it and whatever they called their file.
    assert_eq!(TextId::of(PAGE.as_bytes()), id);
    assert_ne!(
        TextId::of(b"<html><body><p>Ear\xc3\xa1.</p></body></html>"),
        id
    );

    assert_eq!(TextId::parse(id.as_str()), Some(id));
    // Nothing that is not a name is one. A name is the only thing a caller's
    // string becomes, so every shape that would mean something to a path or to
    // a container prefix is refused before it gets that far.
    for refused in [
        "",
        "..",
        "../../etc/passwd",
        "0123456789abcdef0123456789abcde",
        "0123456789abcdef0123456789abcdef0",
        "0123456789ABCDEF0123456789abcdef",
        "0123456789abcdef0123456789abcd/",
        "0123456789abcdef/123456789abcdef",
        "0123456789abcdef%2e%2e6789abcdef",
        "zzzz456789abcdef0123456789abcdef",
    ] {
        assert_eq!(TextId::parse(refused), None, "{refused:?} is not a name");
    }
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn+1/test]
#[tokio::test]
async fn a_stored_text_reads_back_unchanged() {
    let (_root, store) = store();

    let id = store.put(PAGE.as_bytes().to_vec()).await.expect("stored");

    assert_eq!(
        store.get(&id, 64 * 1024).await.expect("read back"),
        PAGE.as_bytes()
    );
}

/// The immutability the stored file's `0400` mode used to carry, translated to
/// a store: a name that is taken is not written over.
// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn+1/test]
#[tokio::test]
async fn a_taken_name_is_never_written_over() {
    let (_root, store) = store();
    let id = store.put(PAGE.as_bytes().to_vec()).await.expect("stored");

    // The same text again is the same name, answered rather than refused —
    // the name is the digest, so what is already there is these bytes.
    assert_eq!(
        store.put(PAGE.as_bytes().to_vec()).await.expect("stored"),
        id
    );
    assert_eq!(
        store.get(&id, 64 * 1024).await.expect("read back"),
        PAGE.as_bytes()
    );

    // And a write of different bytes under a name that is taken is refused by
    // the store itself, which is what the create-only mode buys: no path above
    // this seam can put a teacher's text back as something else.
    let clobber = store
        .store
        .put_opts(
            &ObjectPath::from(id.as_str()),
            PutPayload::from_static(b"<html><body><p>other</p></body></html>"),
            PutOptions {
                mode: PutMode::Create,
                ..PutOptions::default()
            },
        )
        .await;

    assert!(
        matches!(clobber, Err(StoreError::AlreadyExists { .. })),
        "a taken name accepted a second text"
    );
    assert_eq!(
        store.get(&id, 64 * 1024).await.expect("read back"),
        PAGE.as_bytes()
    );
}

/// The local backing keeps the owner-read-only mode the keep directory has
/// always held its texts in, beside the store's own create-only write.
// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn+1/test]
#[tokio::test]
async fn a_locally_stored_text_is_read_only() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().expect("temp dir");
    let directory = root.path().join("keep");
    let store = TextStore::local(&directory).expect("the store opens");

    let id = store.put(PAGE.as_bytes().to_vec()).await.expect("stored");

    let mode = std::fs::metadata(directory.join(id.as_str()))
        .expect("the stored file")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o400);
    assert!(!store.is_durable(), "a directory outlives nothing");
    assert!(store.describe().contains("keep"), "{}", store.describe());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.text-store-fn+1/test]
#[tokio::test]
async fn a_name_that_holds_nothing_is_missing() {
    let (_root, store) = store();
    let absent = TextId::parse("0123456789abcdef0123456789abcdef").expect("a name");

    let error = store
        .get(&absent, 64 * 1024)
        .await
        .expect_err("nothing is stored under that name");

    assert!(
        error.downcast_ref::<Missing>().is_some(),
        "{error:#} does not name a missing text"
    );
}

/// The same cap a fetched page is read under, applied to the store.
// [spec:teaksta:sem:sme.src.main.java.werti.server.wer-ti-servlet.wer-ti-servlet.page-cap-fn+1/test]
#[tokio::test]
async fn a_stored_text_over_the_cap_is_refused() {
    let (_root, store) = store();
    let cap = 1024;
    // Non-ASCII, so an oversized text is reported as oversized rather than as
    // invalid UTF-8 at whichever byte the cap happened to fall inside.
    let id = store
        .put("\u{e1}".repeat(cap).into_bytes())
        .await
        .expect("stored");

    let error = store
        .get(&id, cap)
        .await
        .expect_err("an oversized text is refused");

    let oversized = error
        .downcast_ref::<Oversized>()
        .expect("an oversized text names itself");
    assert_eq!(oversized.cap, cap);
    assert_eq!(oversized.address, id.reference());
    // A text at the cap is still read, so what refuses is the cap and not the
    // reading.
    let under = store
        .put("a".repeat(cap).into_bytes())
        .await
        .expect("stored");
    assert_eq!(store.get(&under, cap).await.expect("read back").len(), cap);
}
