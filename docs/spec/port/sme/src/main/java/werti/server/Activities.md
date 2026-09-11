# sme/src/main/java/werti/server/Activities.java

> [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities+1]
> pub struct Activities {
>   config_map: BTreeMap<String, ActivityConfiguration>,
>   ignored_activities: HashSet<String>,
> }
>
> The registry is built once at startup and read from there. It carries no
> session attribute name, because it is never stashed in a session: there are
> no sessions.

> [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities.activities-fn+2]
> pub fn new(act_dir: &Path, classpath_root: &Path) -> Result<Activities>

> [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn+2]
> Scans a directory of activity folders and builds the registry. Initialises
> `configMap` to an empty string-keyed sorted map, and `ignoredActivities` to a set
> holding exactly one name, the literal `"Conditionals"`.
>
> Lists the entries of `actDir` and, for each entry that is a directory whose
> simple name is not in `ignoredActivities`, parses an activity configuration from
> the file `activity.xml` inside it — path built as the entry's absolute path, the
> platform file separator, then `activity.xml` — and stores it in `configMap` under
> the directory's simple name. Regular files and the `Conditionals` directory are
> skipped silently; the scan is one level deep and does not recurse.
>
> The descriptor root the deployment configured is handed to every activity
> configuration built here, because there is no JVM classpath for a descriptor
> expression to fall back on. The registry neither reads it from the
> environment nor defaults it: it is the caller's, passed through to whatever
> resolves those expressions, so two registries in one process may be built
> against different trees.
>
> No filtering on the activity's own enabled flag happens here, so activities whose
> XML marks them disabled are still loaded and registered. The directory listing
> order is filesystem-dependent, but the sorted map normalises the registry to
> ascending lexicographic order by directory name.
>
> A parse failure for any `activity.xml` (missing file, malformed XML, missing
> expected nodes) propagates out as an I/O error wrapping the underlying cause, and
> aborts construction with the registry partially built. If `actDir` does not exist
> or is not a readable directory the listing yields nothing to iterate and a
> null-pointer error is raised. Nothing is logged and nothing is written to disk.
>
> Port divergence: the ignore list is a constant. The Java builds a set per
> instance and puts one name in it; the set never grows, never varies and is
> read nowhere else, so it is the constant it always was.

> [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities.get-activity-fn]
> public ActivityConfiguration getActivity(String key)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.get-activity-fn]
> Returns the activity configuration registered under the activity name `key`
> (the source directory's simple name, case-sensitive), or null when no activity
> with that name is registered. Hands out the stored instance itself, not a copy,
> so mutations by the caller — including the destructive language-intersection
> performed by the configuration's language accessor — are visible to every later
> lookup of the same key.

> [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities.iterator-fn]
> public Iterator<String> iterator()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.iterator-fn]
> Makes the registry iterable over its activity names by returning an iterator over
> `configMap`'s key set, which yields the names in ascending lexicographic order.
> The iterator is a live view rather than a snapshot: removing through it removes
> the corresponding entry from `configMap`, and modifying the registry during
> iteration invalidates the iterator. Values (the activity configurations
> themselves) are not exposed here; callers pair each name with a separate lookup.

