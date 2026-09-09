# sme/src/main/java/werti/server/FileLocationContextListener.java

> [spec:teaksta:def:sme.src.main.java.werti.server.file-location-context-listener.file-location-context-listener]
> public class FileLocationContextListener implements ServletContextListener

> [spec:teaksta:def:sme.src.main.java.werti.server.file-location-context-listener.file-location-context-listener.context-destroyed-fn]
> public void contextDestroyed(ServletContextEvent servletContextEvent)

> [spec:teaksta:sem:sme.src.main.java.werti.server.file-location-context-listener.file-location-context-listener.context-destroyed-fn]
> Empty. No cleanup is performed on web-application shutdown: the directories
> created at startup are left in place, the context attributes are not removed,
> and uploaded files accumulated under the temporary upload directory are never
> deleted.

> [spec:teaksta:def:sme.src.main.java.werti.server.file-location-context-listener.file-location-context-listener.context-initialized-fn]
> public void contextInitialized(ServletContextEvent servletContextEvent)

> [spec:teaksta:sem:sme.src.main.java.werti.server.file-location-context-listener.file-location-context-listener.context-initialized-fn]
> Runs once at web-application startup. Takes the `ServletContext` from the
> event, then creates the three upload directories and publishes each one into
> the context under two attribute names — a `java.io.File` and the raw path
> string — so that `UploadDownloadFileServlet` can find them.
>
> First generates a 10-character random alphanumeric string and prints
> `rnd_name=` plus that string to standard output. The value is used for nothing
> else; it is dead.
>
> Reads the context init parameter `files_dir` (declared in `web.xml`, value
> `/home/teaksta/fileUpload/`), constructs a `File` directly from that path with
> no application-root prefixing, and calls `mkdirs()` on it if it does not
> already exist. Prints the fixed line
> `File Directory created to be used for storing files` to standard output —
> unconditionally, whether or not anything was created. Sets the context
> attribute `FILES_DIR_FILE` to the `File` and `FILES_DIR` to the path string.
>
> Repeats the same sequence, without the print, for the init parameter
> `files_prm_dir` (value `/home/teaksta/fileUpload/prm/`, the directory for files
> the uploader gave permission to keep), setting `FILES_PRM_DIR_FILE` and
> `FILES_PRM_DIR`; and again for `files_tmp_dir` (value
> `/home/teaksta/fileUpload/tmp/`, the directory for files to be discarded),
> setting `FILES_TMP_DIR_FILE` and `FILES_TMP_DIR`.
>
> Returns void. The boolean result of each `mkdirs()` is ignored, so a directory
> that cannot be created is not reported and only surfaces later as an upload
> failure. A missing init parameter yields null and the `File` constructor then
> throws a `NullPointerException` out of the listener, aborting context startup.
> The fourth upload-related init parameter `files_anl_dir`
> (`/home/teaksta/analyzedTexts/`, the CAS cache directory) is not handled here.

