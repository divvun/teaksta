# sme/src/main/java/werti/server/Activities.java

> [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities]
> public class Activities implements Iterable<String> {
>   public static final String ATT_NAME = "werti.activities";
>   private TreeMap<String, ActivityConfiguration> configMap;
>   private Set<String> ignoredActivities;
> }

> [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities.activities-fn]
> public Activities(File actDir) throws IOException

> [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities.get-activity-fn]
> public ActivityConfiguration getActivity(String key)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.get-activity-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities.iterator-fn]
> public Iterator<String> iterator()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.iterator-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

