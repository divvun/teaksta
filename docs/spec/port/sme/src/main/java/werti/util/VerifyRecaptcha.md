# sme/src/main/java/werti/util/VerifyRecaptcha.java

> [spec:teaksta:def:sme.src.main.java.werti.util.verify-recaptcha.verify-recaptcha]
> public class VerifyRecaptcha {
>   public static final String url = "https://www.google.com/recaptcha/api/siteverify";
>   private final static String USER_AGENT = "Mozilla/5.0";
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.verify-recaptcha.verify-recaptcha.verify-fn]
> public static boolean verify(String gRecaptchaResponse, String secret) throws IOException

> [spec:teaksta:sem:sme.src.main.java.werti.util.verify-recaptcha.verify-recaptcha.verify-fn]
> Validates a Google reCAPTCHA response token against the siteverify endpoint and
> returns whether Google accepted it.
>
> If `gRecaptchaResponse` is null or the empty string, returns false without any
> network access. `secret` is not checked.
>
> Otherwise opens an HTTPS connection to the class constant
> `https://www.google.com/recaptcha/api/siteverify`, sets the request method to
> `POST`, the header `User-Agent` to `Mozilla/5.0` (the `USER_AGENT` constant) and
> the header `Accept-Language` to `en-US,en;q=0.5`. Builds the body by plain
> string concatenation as `secret=<secret>&response=<gRecaptchaResponse>` with no
> percent-encoding of either value, enables output on the connection, and writes
> the body with `DataOutputStream.writeBytes` — which emits only the low byte of
> each character, so any non-ASCII character is mangled — then flushes and closes
> the stream.
>
> Reads the HTTP status code, then prints to standard output: a blank line
> followed by `Sending 'POST' request to URL : ` plus the endpoint, then
> `Post parameters : ` plus the body, then `Response Code : ` plus the status
> code. Reads the response body line by line from the connection's input stream
> and appends the lines to a buffer with no separator between them, closes the
> reader, and prints the accumulated body to standard output.
>
> Parses the accumulated body as a JSON object and returns the boolean value of
> its `success` member.
>
> Every failure inside the request — connection error, non-2xx status making
> `getInputStream` throw, malformed JSON, or a missing/non-boolean `success`
> member — is caught by a blanket `catch (Exception)` which prints the stack trace
> to standard error and returns false. The method is declared `throws IOException`
> but no exception ever escapes it.
>
> Quirk: the shared secret is echoed to standard output on every call as part of
> the `Post parameters` line, so it lands in the servlet container's log. Quirk:
> no connect or read timeout is set, so a hung endpoint blocks the calling request
> thread indefinitely. Quirk: the optional client IP (`remoteip`) parameter is
> never sent.

