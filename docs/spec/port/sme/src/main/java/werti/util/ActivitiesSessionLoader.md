# sme/src/main/java/werti/util/ActivitiesSessionLoader.java

> [spec:teaksta:def:sme.src.main.java.werti.util.activities-session-loader.activities-session-loader]
> public class ActivitiesSessionLoader

> [spec:teaksta:def:sme.src.main.java.werti.util.activities-session-loader.activities-session-loader.create-activities-in-session-fn]
> public static Activities createActivitiesInSession(HttpServletRequest req) throws IOException

> [spec:teaksta:sem:sme.src.main.java.werti.util.activities-session-loader.activities-session-loader.create-activities-in-session-fn]
> Lazily installs the activity registry into the HTTP session and returns it.
> Obtains the request's session, creating one if the request has none, then reads
> the session attribute named by the registry's attribute-name constant, the
> literal string `"werti.activities"`, and casts it to an activities registry.
>
> If that attribute is absent (null), builds a fresh registry by scanning the
> webapp-relative directory `/activities` resolved to a real filesystem path
> through the servlet context, stores the new registry back into the session under
> `"werti.activities"`, and uses it. If the attribute was already present, it is
> returned as-is with no rescan. Either way the registry is returned.
>
> An I/O error from scanning the activity directory or parsing any `activity.xml`
> propagates to the caller and leaves the session attribute unset. If the session
> attribute holds an object of some other type, the cast raises a class-cast error.
> If the webapp is served unexpanded so the real path cannot be resolved, the path
> is null and a null-pointer error is raised while building the directory handle.
>
> Quirk: despite the "create a fresh registry" framing, the result is cached per
> session for the session's lifetime, so edits to activity XML on disk are not
> picked up until the session ends. The check-then-store is not synchronised, so
> two concurrent requests on one session can each scan the directory and the later
> store wins.

