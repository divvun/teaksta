# sme/src/main/java/werti/util/VerifyRecaptcha.java

> [spec:teaksta:def:sme.src.main.java.werti.util.verify-recaptcha.verify-recaptcha]
> public class VerifyRecaptcha {
>   public static final String url = "https://www.google.com/recaptcha/api/siteverify";
>   private final static String USER_AGENT = "Mozilla/5.0";
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.verify-recaptcha.verify-recaptcha.verify-fn]
> public static boolean verify(String gRecaptchaResponse, String secret) throws IOException

> [spec:teaksta:sem:sme.src.main.java.werti.util.verify-recaptcha.verify-recaptcha.verify-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

