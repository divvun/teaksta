# sme/src/main/java/werti/uima/ae/GiellateknoTokenizer.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer]
> public class GiellateknoTokenizer extends JCasAnnotator_ImplBase {
>   private static Map<String, TokenizerME> tokenizers;
>   private static final Logger log = LogManager.GetLogger(GiellateknoTokenizer.class);
>   private static final String toolsDir = Constants.tools_Dir;
>   private static final String abbrDir = Constants.abbr_Dir;
>   private static final String preprocessCmd = toolsDir + "preprocess --abbr=" + abbrDir + "abbr.txt";
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string]
> public class ExtCommandConsume2String implements Runnable {
>   private BufferedReader reader;
>   private boolean finished;
>   private String buffer;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.ext-command-consume2-string-fn]
> public ExtCommandConsume2String(BufferedReader reader)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.ext-command-consume2-string-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.get-buffer-fn]
> public String getBuffer()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.get-buffer-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.is-done-fn]
> public boolean isDone()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.is-done-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.run-fn]
> public void run()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.ext-command-consume2-string.run-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.initialize-fn]
> @Override public void initialize(UimaContext aContext) throws ResourceInitializationException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.initialize-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn]
> @SuppressWarnings("unchecked") @Override public void process(JCas jcas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.giellatekno-tokenizer.giellatekno-tokenizer.process-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

