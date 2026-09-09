# sme/src/main/java/werti/uima/ae/Vislcg3Annotator.java

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator]
> public class Vislcg3Annotator extends JCasAnnotator_ImplBase {
>   private static final Logger log = LogManager.GetLogger(Vislcg3Annotator.class);
>   private final String CGSentenceBoundaryToken = ".";
>   private String vislcg3Loc = Constants.vislcg3_Loc;
>   private String vislcg3DisGrammarLoc = Constants.vislcg3_DisGrammarLoc;
>   private String vislcg3SyntGrammarLoc = Constants.vislcg3_SyntGrammarLoc;
>   private final String preprocessLoc = Constants.preprocess_Loc;
>   private final String abbr = Constants.abbr_file;
>   private final String lookupLoc = Constants.lookup_Loc;
>   private final String lookupFlags = Constants.lookup_Flags;
>   private final String fstLoc = Constants.an_FST;
>   private final String lookup2cgLoc = Constants.lookup_2cgLoc;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.copy-fn]
> private void copy(Token source, CGToken target)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.copy-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger]
> public class ExtCommandConsume2Logger implements Runnable {
>   private BufferedReader reader;
>   private String msgPrefix;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.ext-command-consume2-logger-fn]
> public ExtCommandConsume2Logger(BufferedReader reader, String msgPrefix)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.ext-command-consume2-logger-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.run-fn]
> public void run()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-logger.run-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string]
> public class ExtCommandConsume2String implements Runnable {
>   private BufferedReader reader;
>   private boolean finished;
>   private String buffer;
> }

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.ext-command-consume2-string-fn]
> public ExtCommandConsume2String(BufferedReader reader)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.ext-command-consume2-string-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.get-buffer-fn]
> public String getBuffer()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.get-buffer-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.is-done-fn]
> public boolean isDone()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.is-done-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.run-fn]
> public void run()

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.ext-command-consume2-string.run-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn]
> private List<CGToken> parseCGOutput(String cgOutput, JCas jcas)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.parse-cg-output-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn]
> @Override public void process(JCas jcas) throws AnalysisEngineProcessException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.process-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.run-fst-cg-fn]
> private String runFST_CG(String input) throws IOException,InterruptedException

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.run-fst-cg-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

> [spec:teaksta:def:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn]
> private String toCG3Input(List<Token> tokenList, List<SentenceAnnotation> sentList)

> [spec:teaksta:sem:sme.src.main.java.werti.uima.ae.vislcg3-annotator.vislcg3-annotator.to-cg3-input-fn]
> TODO(sem): what this does, step by step — precisely enough to
> re-implement from this rule alone, without reading the source.

