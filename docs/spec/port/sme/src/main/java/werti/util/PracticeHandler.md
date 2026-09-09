# sme/src/main/java/werti/util/PracticeHandler.java

> [spec:teaksta:def:sme.src.main.java.werti.util.practice-handler.practice-handler]
> public class PracticeHandler {
>   private static final Logger log = LogManager.GetLogger(PracticeHandler.class);
>   private PostRequest requestInfo;
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.practice-handler.practice-handler.practice-handler-fn]
> public PracticeHandler(PostRequest aRequestInfo)

> [spec:teaksta:sem:sme.src.main.java.werti.util.practice-handler.practice-handler.practice-handler-fn]
> Stores the supplied `PostRequest` in the instance field `requestInfo` and does
> nothing else. No null check, no defensive copy, no normalisation of the
> request's fields, and no logging. The `PostRequest` is retained by reference,
> so later mutation by the caller is visible through the handler.

> [spec:teaksta:def:sme.src.main.java.werti.util.practice-handler.practice-handler.process-fn]
> public String process()

> [spec:teaksta:sem:sme.src.main.java.werti.util.practice-handler.practice-handler.process-fn]
> Unconditionally returns the empty string. It never reads `requestInfo`, never
> touches the logger, performs no I/O and has no side effects. Practice-mode
> response handling is unimplemented: callers that route a request to
> `PracticeHandler` receive an empty body.
>
> Quirk: the class's static logger is obtained via `LogManager.GetLogger` (capital
> `G`), which is not a real log4j2 method name — a residue of an unfinished
> log4j migration that prevents the class from compiling as written.

