# sme/src/main/java/werti/util/PostRequest.java

> [spec:teaksta:def:sme.src.main.java.werti.util.post-request.post-request]
> public class PostRequest {
>   public String type;
>   public String url;
>   public String language;
>   public String topic;
>   public String activity;
>   public String document;
>   public String version;
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.post-request.post-request.to-string-fn]
> public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.util.post-request.post-request.to-string-fn]
> Renders the request record as a multi-line debug string by appending, in this
> exact order, to a single buffer: the literal `PostRequest(`, then for each
> field a newline followed by two spaces, the field name, ` = ` and the field's
> value — `type`, `url`, `language`, `topic`, `activity`, `document`, `version`
> in that order — and finally a newline followed by `)`. The full template is
> `PostRequest(\n  type = T\n  url = U\n  language = L\n  topic = O\n  activity = A\n  document = D\n  version = V\n)`.
>
> Field values are appended with Java string conversion, so a null field is
> rendered as the four-character literal `null` rather than an empty string. No
> escaping, truncation or trimming is applied, so a large `document` body is
> embedded verbatim. The method has no side effects and reads only the instance
> fields.

