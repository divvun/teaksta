# sme/src/main/java/werti/util/Resources.java

> [spec:teaksta:def:sme.src.main.java.werti.util.resources.resources]
> public class Resources {
>   private static final Logger log = LogManager.GetLogger(Resources.class);
> }

> [spec:teaksta:def:sme.src.main.java.werti.util.resources.resources.get-resource-fn]
> private static final Object getResource(URL path) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.util.resources.resources.get-resource-fn]
> Java-deserialises a single object from the byte stream behind `path` and
> returns it. This is the shared back end of the two public `getResource`
> overloads (one resolving through a `ServletContext`, one resolving a
> `file://`-relative URL), both of which log the resolved path at debug level
> before delegating here.
>
> Steps: open `path.openStream()` to get an `InputStream`; wrap it in an
> `ObjectInputStream` (which reads and validates the serialization stream header
> immediately); call `readObject()`; close the `ObjectInputStream`, then close
> the `InputStream`; return the deserialised object as `Object`. Callers are
> responsible for casting it to the model type they expect.
>
> Error handling, all funnelled through the `noAccess` factory:
> a `ClassNotFoundException` from `readObject` becomes a
> `ResourceInitializationException` with reason
> `"The class of the model object is unknown. Path: " + path` (the URL's full
> `toString`, not just its path component) and the original exception as cause;
> any `IOException` escaping the whole block — from `openStream`, from the
> header check, or from `readObject` — becomes one with reason
> `path.getPath()`; a `NullPointerException` becomes one with reason
> `"Couldn't load path: " + path.getPath()`.
>
> Two nested `finally` blocks re-close the `ObjectInputStream` and then the
> `InputStream`, each converting an `IOException` raised while closing into
> `noAccess(path.getPath(), ioe)`. Because both streams were already closed on
> the success path, these are normally redundant second closes.
>
> Quirk: those `finally` blocks run on the success path too, so an `IOException`
> while re-closing discards the already-computed return value and turns a
> successful read into a failure. Quirk: an exception thrown from a `finally`
> also replaces and masks any in-flight `ClassNotFoundException` conversion.
> Quirk: `ResourceInitializationException` is not an `IOException`, so the
> exceptions raised inside the `finally` blocks bypass the outer catch clauses
> and propagate unchanged. Quirk: the `NullPointerException` handler dereferences
> `path` to build its message, so when `path` itself is null — which is exactly
> what makes `path.openStream()` throw — the handler throws a fresh
> `NullPointerException` out of the catch block instead of the intended
> `ResourceInitializationException`.

> [spec:teaksta:def:sme.src.main.java.werti.util.resources.resources.no-access-fn]
> private static final ResourceInitializationException noAccess(final String reason, final Exception exception)

> [spec:teaksta:sem:sme.src.main.java.werti.util.resources.resources.no-access-fn]
> Boiler-plate factory that builds — but does not throw — a UIMA
> `ResourceInitializationException` describing a failed resource read.
>
> Wraps `reason` in a one-element `Object[]` message-argument array, constructs
> `new ResourceInitializationException(ResourceInitializationException.COULD_NOT_ACCESS_DATA,
> args, exception)` so that the message key is the standard
> `COULD_NOT_ACCESS_DATA` key, `reason` is its single substitution argument, and
> `exception` is the chained cause, then returns that object to the caller.
>
> No logging, no side effects. Every call site is of the form
> `throw noAccess(...)`, so the exception is always thrown by the caller rather
> than here; returning it instead of throwing is purely to keep the throw sites
> short.

