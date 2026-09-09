# sme/src/main/java/werti/util/DummyException.java

> [spec:teaksta:def:sme.src.main.java.werti.util.dummy-exception.dummy-exception]
> public class DummyException extends Exception

> [spec:teaksta:def:sme.src.main.java.werti.util.dummy-exception.dummy-exception.dummy-exception-fn]
> public DummyException()

> [spec:teaksta:sem:sme.src.main.java.werti.util.dummy-exception.dummy-exception.dummy-exception-fn]
> The no-argument constructor: delegates to `Exception()` and does nothing else,
> producing an instance with a null detail message and no cause, but with a
> stack trace filled in at construction as for any `Throwable`.
>
> `DummyException` is a checked exception used purely as a control-flow signal —
> it is thrown to force execution out of a block and into an enclosing `finally`
> or `catch`, never to report a real error, so it carries no payload. Three
> further constructors exist alongside this one, taking a message, a cause, or
> both, and each simply forwards to the matching `Exception` constructor.
>
> Quirk: the class extends `Exception` without declaring a `serialVersionUID`.

