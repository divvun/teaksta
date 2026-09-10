use super::*;

const BODY_PREFIX: &str = "<html xmlns='http://www.w3.org/1999/xhtml' xml:lang='en' lang='sme'><head><meta charset='utf-8'></head><body>";
const BODY_SUFFIX: &str = "</body></html>";
const HOST_NAME: &str = "http://gtoahpa-01.uit.no/konteaksta";

fn error_block(message: &str) -> String {
    format!(
        "<br><br><br><center><h1 style='color:#144ea6;'>{}</h1></center><center><a href={}>Ruovttoluotta</a></center>",
        message, HOST_NAME
    )
}

fn error_page(message: &str) -> String {
    format!("{}{}{}", BODY_PREFIX, error_block(message), BODY_SUFFIX)
}

fn failure_page(cause: &str) -> String {
    format!(
        "{}Exception in uploading file. Cause: {}{}",
        BODY_PREFIX, cause, BODY_SUFFIX
    )
}

fn form_field(name: &str, value: &str) -> FileItem {
    FileItem {
        field_name: name.to_string(),
        file_name: None,
        content_type: None,
        content: value.as_bytes().to_vec(),
    }
}

fn file_part(name: &str, file_name: &str, content: &str) -> FileItem {
    FileItem {
        field_name: name.to_string(),
        file_name: Some(file_name.to_string()),
        content_type: Some("text/html".to_string()),
        content: content.as_bytes().to_vec(),
    }
}

fn multipart_request(items: Vec<FileItem>) -> HttpServletRequest {
    HttpServletRequest {
        method: "POST".to_string(),
        content_type: Some("multipart/form-data; boundary=----teaksta".to_string()),
        items,
    }
}

fn initialised_servlet(context: ServletContext) -> UploadDownloadFileServlet {
    let mut servlet = UploadDownloadFileServlet::new();
    servlet.init(context).expect("init never fails");
    servlet
}

fn context_with_directories(root: &Path) -> (ServletContextEvent, String, String, String) {
    let files_dir = format!("{}/fileUpload/", root.display());
    let files_prm_dir = format!("{}/fileUpload/prm/", root.display());
    let files_tmp_dir = format!("{}/fileUpload/tmp/", root.display());

    let mut event = ServletContextEvent::default();
    let context = event.get_servlet_context_mut();
    context
        .init_parameters
        .insert("files_dir".to_string(), files_dir.clone());
    context
        .init_parameters
        .insert("files_prm_dir".to_string(), files_prm_dir.clone());
    context
        .init_parameters
        .insert("files_tmp_dir".to_string(), files_tmp_dir.clone());

    (event, files_dir, files_prm_dir, files_tmp_dir)
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.init-fn/test]
#[test]
fn init_uses_published_upload_dir_as_repository() {
    let upload_dir = tempfile::tempdir().expect("temp dir");
    let mut context = ServletContext::default();
    context.set_attribute(
        "FILES_DIR_FILE",
        Attribute::File(upload_dir.path().to_path_buf()),
    );

    let mut servlet = UploadDownloadFileServlet::new();
    assert!(servlet.get_servlet_context().is_err());

    servlet.init(context).expect("init never fails");

    assert!(servlet.get_servlet_context().is_ok());
    let factory = &servlet
        .uploader
        .as_ref()
        .expect("uploader")
        .file_item_factory;
    assert_eq!(factory.repository.as_deref(), Some(upload_dir.path()));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.init-fn/test]
#[test]
fn init_keeps_default_repository_when_attribute_absent() {
    let mut servlet = UploadDownloadFileServlet::new();

    servlet.init(ServletContext::default()).expect("init");

    let factory = &servlet
        .uploader
        .as_ref()
        .expect("uploader")
        .file_item_factory;
    assert!(factory.repository.is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.init-fn/test]
#[test]
fn init_keeps_default_repository_when_attribute_not_file() {
    let mut context = ServletContext::default();
    context.set_attribute(
        "FILES_DIR_FILE",
        Attribute::Text("/home/teaksta/fileUpload/".to_string()),
    );

    let mut servlet = UploadDownloadFileServlet::new();
    servlet.init(context).expect("init");

    let factory = &servlet
        .uploader
        .as_ref()
        .expect("uploader")
        .file_item_factory;
    assert!(factory.repository.is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn/test]
#[test]
fn check_meta_data_matches_detected_type_substring() {
    let dir = tempfile::tempdir().expect("temp dir");
    let stored = dir.path().join("Ab3xY9Qw7z");
    std::fs::write(
        &stored,
        "<html><head><title>Oahpa</title></head><body>Sámegiella</body></html>",
    )
    .expect("write");

    assert!(check_meta_data(&stored, "text/html").expect("detected"));
    assert!(!check_meta_data(&stored, "application/xhtml+xml").expect("detected"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn/test]
#[test]
fn check_meta_data_tells_xhtml_apart_from_html() {
    let dir = tempfile::tempdir().expect("temp dir");
    let stored = dir.path().join("Qq1Ww2Ee3R");
    std::fs::write(
        &stored,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<html xmlns=\"http://www.w3.org/1999/xhtml\" lang=\"sme\"><body>Sámi</body></html>",
    )
    .expect("write");

    assert!(check_meta_data(&stored, "application/xhtml+xml").expect("detected"));
    assert!(!check_meta_data(&stored, "text/html").expect("detected"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn/test]
#[test]
fn check_meta_data_false_when_file_unreadable() {
    let dir = tempfile::tempdir().expect("temp dir");
    let missing = dir.path().join("never-written");

    assert!(!check_meta_data(&missing, "text/html").expect("read failure is false"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.check-meta-data-fn/test]
#[test]
fn check_meta_data_raises_when_type_undetected() {
    let dir = tempfile::tempdir().expect("temp dir");
    let stored = dir.path().join("Zz9Yy8Xx7W");
    std::fs::write(&stored, b"").expect("write");

    let error = check_meta_data(&stored, "text/html").expect_err("null content type");

    assert!(error.to_string().contains("NullPointerException"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn/test]
#[test]
fn post_rejects_non_multipart_before_writing() {
    let servlet = initialised_servlet(ServletContext::default());
    let request = HttpServletRequest {
        method: "POST".to_string(),
        content_type: Some("application/x-www-form-urlencoded".to_string()),
        items: Vec::new(),
    };
    let mut response = HttpServletResponse::default();

    let error = servlet
        .handle_post(&request, &mut response)
        .expect_err("not multipart");

    assert_eq!(error.to_string(), "Content type is not multipart/form-data");
    assert!(response.body.is_empty());
    assert!(response.content_type.is_none());
    assert!(response.character_encoding.is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn/test]
#[test]
fn post_reports_null_cause_for_empty_part_list() {
    let servlet = initialised_servlet(ServletContext::default());
    let request = multipart_request(Vec::new());
    let mut response = HttpServletResponse::default();

    servlet
        .handle_post(&request, &mut response)
        .expect("handled");

    assert_eq!(response.body, failure_page("null"));
    assert_eq!(response.content_type.as_deref(), Some("text/html"));
    assert_eq!(response.character_encoding.as_deref(), Some("UTF-8"));
    assert!(response.redirect.is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn/test]
#[test]
fn post_asks_for_file_when_upload_part_empty() {
    let servlet = initialised_servlet(ServletContext::default());
    let request = multipart_request(vec![
        form_field("activity_up", "Click"),
        form_field("enhancement_upload", "Nouns"),
        form_field("save_options", "save"),
        file_part("file_upload", "empty.html", ""),
    ]);
    let mut response = HttpServletResponse::default();

    servlet
        .handle_post(&request, &mut response)
        .expect("handled");

    assert_eq!(response.body, error_page("Vajálduhttet sáddet fiilla!"));
    assert!(response.redirect.is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn/test]
#[test]
fn post_reports_captcha_error_when_none_sent() {
    let servlet = initialised_servlet(ServletContext::default());
    let request = multipart_request(vec![
        form_field("activity_up", "Click"),
        form_field("enhancement_upload", "Nouns"),
        file_part(
            "file_upload",
            "page.html",
            "<html><body>Sámegiella</body></html>",
        ),
    ]);
    let mut response = HttpServletResponse::default();

    servlet
        .handle_post(&request, &mut response)
        .expect("handled");

    assert_eq!(response.body, error_page("Vajálduhttet captcha!"));
    assert!(response.redirect.is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn/test]
#[test]
fn post_treats_first_part_as_upload_without_file() {
    let servlet = initialised_servlet(ServletContext::default());
    let with_content = multipart_request(vec![
        form_field("activity_up", "Click"),
        form_field("enhancement_upload", "Nouns"),
    ]);
    let mut response = HttpServletResponse::default();

    servlet
        .handle_post(&with_content, &mut response)
        .expect("handled");

    assert_eq!(response.body, error_page("Vajálduhttet captcha!"));

    let with_empty_first_part = multipart_request(vec![
        form_field("activity_up", ""),
        form_field("enhancement_upload", "Nouns"),
    ]);
    let mut response = HttpServletResponse::default();

    servlet
        .handle_post(&with_empty_first_part, &mut response)
        .expect("handled");

    assert_eq!(response.body, error_page("Vajálduhttet sáddet fiilla!"));
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn/test]
#[test]
fn post_reads_secret_key_but_rejects_empty_token() {
    let dir = tempfile::tempdir().expect("temp dir");
    let secret_key = dir.path().join("secret_key.do.not.check.in");
    std::fs::write(&secret_key, "sh4red-s3cret\nignored second line\n").expect("write");

    let mut context = ServletContext::default();
    context.init_parameters.insert(
        "secret_key".to_string(),
        secret_key.to_string_lossy().into_owned(),
    );
    let servlet = initialised_servlet(context);

    let request = multipart_request(vec![
        form_field("activity_up", "Click"),
        file_part(
            "file_upload",
            "page.html",
            "<html><body>Sámegiella</body></html>",
        ),
        form_field("g-recaptcha-response", ""),
    ]);
    let mut response = HttpServletResponse::default();

    servlet
        .handle_post(&request, &mut response)
        .expect("handled");

    assert_eq!(response.body, error_page("Vajálduhttet captcha!"));
    assert!(response.redirect.is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.upload-download-file-servlet.upload-download-file-servlet.do-post-fn/test]
#[test]
fn post_fails_when_secret_key_parameter_missing() {
    let servlet = initialised_servlet(ServletContext::default());
    let request = multipart_request(vec![
        file_part(
            "file_upload",
            "page.html",
            "<html><body>Sámegiella</body></html>",
        ),
        form_field("g-recaptcha-response", "a-token-the-client-sent"),
    ]);
    let mut response = HttpServletResponse::default();

    servlet
        .handle_post(&request, &mut response)
        .expect("handled");

    assert_eq!(response.body, failure_page("null"));
    assert!(response.redirect.is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.file-location-context-listener.file-location-context-listener.context-initialized-fn/test]
#[test]
fn context_initialized_creates_dirs_publishes_both_forms() {
    let root = tempfile::tempdir().expect("temp dir");
    let (mut event, files_dir, files_prm_dir, files_tmp_dir) =
        context_with_directories(root.path());

    FileLocationContextListener::new()
        .context_initialized(&mut event)
        .expect("startup");

    assert!(root.path().join("fileUpload").is_dir());
    assert!(root.path().join("fileUpload/prm").is_dir());
    assert!(root.path().join("fileUpload/tmp").is_dir());

    let context = event.get_servlet_context();
    for (file_name, text_name, raw) in [
        ("FILES_DIR_FILE", "FILES_DIR", &files_dir),
        ("FILES_PRM_DIR_FILE", "FILES_PRM_DIR", &files_prm_dir),
        ("FILES_TMP_DIR_FILE", "FILES_TMP_DIR", &files_tmp_dir),
    ] {
        assert!(raw.ends_with('/'));
        let published = context.get_attribute(file_name).expect(file_name);
        assert_eq!(published.as_file(), Some(new_file(raw).as_path()));
        // The `File` form drops the trailing separator; the string form
        // is the init parameter verbatim.
        assert_eq!(published.to_java_string(), raw.trim_end_matches('/'));
        assert_eq!(
            context
                .get_attribute(text_name)
                .expect(text_name)
                .to_java_string(),
            *raw
        );
        assert!(
            context
                .get_attribute(text_name)
                .unwrap()
                .as_file()
                .is_none()
        );
    }

    assert!(context.get_attribute("FILES_ANL_DIR").is_none());
    assert!(context.get_attribute("FILES_ANL_DIR_FILE").is_none());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.file-location-context-listener.file-location-context-listener.context-initialized-fn/test]
#[test]
fn context_initialized_keeps_directories_that_already_exist() {
    let root = tempfile::tempdir().expect("temp dir");
    let (mut event, files_dir, _prm, _tmp) = context_with_directories(root.path());
    std::fs::create_dir_all(new_file(&files_dir)).expect("pre-existing directory");
    let marker = root.path().join("fileUpload/already-there");
    std::fs::write(&marker, b"kept").expect("write");

    FileLocationContextListener::new()
        .context_initialized(&mut event)
        .expect("startup");

    assert_eq!(std::fs::read(&marker).expect("read"), b"kept");
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.file-location-context-listener.file-location-context-listener.context-initialized-fn/test]
#[test]
fn context_initialized_aborts_midway_on_missing_parameter() {
    let root = tempfile::tempdir().expect("temp dir");
    let files_dir = format!("{}/fileUpload/", root.path().display());
    let mut event = ServletContextEvent::default();
    event
        .get_servlet_context_mut()
        .init_parameters
        .insert("files_dir".to_string(), files_dir);

    let error = FileLocationContextListener::new()
        .context_initialized(&mut event)
        .expect_err("missing files_prm_dir");

    assert!(error.to_string().contains("files_prm_dir"));
    let context = event.get_servlet_context();
    assert!(context.get_attribute("FILES_DIR").is_some());
    assert!(context.get_attribute("FILES_DIR_FILE").is_some());
    assert!(context.get_attribute("FILES_PRM_DIR").is_none());
    assert!(context.get_attribute("FILES_TMP_DIR").is_none());
    assert!(root.path().join("fileUpload").is_dir());
}

// [spec:teaksta:sem:sme.src.main.java.werti.server.file-location-context-listener.file-location-context-listener.context-destroyed-fn/test]
#[test]
fn context_destroyed_removes_nothing() {
    let root = tempfile::tempdir().expect("temp dir");
    let (mut event, _files_dir, _prm, _tmp) = context_with_directories(root.path());
    let listener = FileLocationContextListener::new();
    listener.context_initialized(&mut event).expect("startup");
    let leftover = root.path().join("fileUpload/tmp/Ab3xY9Qw7z");
    std::fs::write(&leftover, b"an upload nobody cleans up").expect("write");
    let attributes_before = event.get_servlet_context().attributes.len();

    listener.context_destroyed(&event);

    assert_eq!(
        event.get_servlet_context().attributes.len(),
        attributes_before
    );
    assert!(
        event
            .get_servlet_context()
            .get_attribute("FILES_TMP_DIR_FILE")
            .is_some()
    );
    assert!(leftover.is_file());
    assert!(root.path().join("fileUpload/prm").is_dir());
}
