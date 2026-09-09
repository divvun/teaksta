# sme/src/main/java/werti/server/ActivityConfiguration.java

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration]
> public class ActivityConfiguration {
>   public static final String CLIENT_PREFIX = "client";
>   public static final String PRE_PREFIX = "pre";
>   public static final String POST_PREFIX = "post";
>   public static final String LANG_PREFIX = "lang";
>   public static final String ACT_PLACEHOLDER = "_ACT_";
>   private String actbaseDir;
>   private HashMap<String, URL> preDesc;
>   private HashMap<String, URL> postDesc;
>   private HashMap<String, HashMap<String, ConfigValue>> clientConfig;
>   private HashMap<String, HashMap<String, ConfigValue>> serverPreConfig;
>   private HashMap<String, HashMap<String, ConfigValue>> serverPostConfig;
>   private final String nl = System.getProperty("line.separator");
>   private boolean isEnabled;
>   private String name;
> }

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.activity-configuration-fn]
> public ActivityConfiguration(File xmlActivityConfig) throws IOException

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.activity-configuration-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value]
> public class ConfigValue {
>   private String value;
>   private boolean readOnly;
> }

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.config-value-fn]
> public ConfigValue(String value, boolean readOnly)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.config-value-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.get-value-fn]
> public String getValue()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.get-value-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.is-read-only-fn]
> public boolean isReadOnly()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.is-read-only-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.to-string-fn]
> public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config-value.to-string-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config2-props-fn]
> private Properties config2Props(String lang, HashMap<String,HashMap<String,ConfigValue>> conf)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.config2-props-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-client-value-fn]
> public String getClientValue(String lang, String key)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-client-value-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-languages-fn]
> public Set<String> getLanguages()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-languages-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-name-fn]
> public String getName()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-name-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-post-desc-fn]
> public URL getPostDesc(String lang)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-post-desc-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-pre-desc-fn]
> public URL getPreDesc(String lang)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-pre-desc-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-config-as-prop-fn]
> public Properties getServerPostConfigAsProp(String lang)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-config-as-prop-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-keys-fn]
> public Set<String> getServerPostKeys()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-keys-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-value-fn]
> public String getServerPostValue(String lang, String key)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-post-value-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-config-as-prop-fn]
> public Properties getServerPreConfigAsProp(String lang)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-config-as-prop-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-keys-fn]
> public Set<String> getServerPreKeys()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-keys-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-value-fn]
> public String getServerPreValue(String lang, String key)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-server-pre-value-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-value-fn]
> private String getValue(String lang, String key, HashMap<String,HashMap<String,ConfigValue>> conf)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.get-value-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.is-enabled-fn]
> public boolean isEnabled()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.is-enabled-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.load-from-xml-fn]
> private void loadFromXml(InputStream is) throws ParserConfigurationException, SAXException, IOException, XPathExpressionException

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.load-from-xml-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.main-fn]
> public static void main(String[] args) throws IOException

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.main-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.read-xml-conf-entries-fn]
> private HashMap<String,ConfigValue> readXmlConfEntries(Node n)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.read-xml-conf-entries-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-client-value-fn]
> public boolean setClientValue(String lang, String key, String value)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-client-value-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-post-value-fn]
> public boolean setServerPostValue(String lang, String key, String value)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-post-value-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-pre-value-fn]
> public boolean setServerPreValue(String lang, String key, String value)

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-server-pre-value-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-value-fn]
> private boolean setValue(String lang, String key, String value, HashMap<String,HashMap<String,ConfigValue>> conf )

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.set-value-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.server.activity-configuration.activity-configuration.to-string-fn]
> public String toString()

> [spec:teaksta:sem:sme.src.main.java.werti.server.activity-configuration.activity-configuration.to-string-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

