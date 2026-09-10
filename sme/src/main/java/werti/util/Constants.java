package werti.util;

//Class containing all paths.
//To work locally uncomment (and change user name) the 'LOCAL PATHS' section and comment the 'GTOAHPA PATHS' section.

public final class Constants {
  //LOCAL PATHS:
  /*
    //To work locally insert your username here, then everything should work fine
    public static final String user_name = "car010";

    //If something goes wrong check where you have lookup, preprocess, vislcg3 and lookup2cg (which PROGRAM):
    //public static final String lookup_Loc = "/usr/local/bin/lookup";
    public static final String lookup_Loc = "/usr/local/bin/hfst-optimized-lookup"; // for new fst
    public static String preprocess_Loc = "/Users/"+user_name+"/main/giella-core/scripts/preprocess";
    public static String vislcg3_Loc = "/usr/local/bin/vislcg3";
    public static String lookup2cg_loc = "/Users/"+user_name+"/main/giella-core/scripts/lookup2cg";

    //If you still have problems, check where you have the files in use:
    //abbr.txt and corr.txt:
    public static final String abbr_Dir = "/Users/"+user_name+"/main/langs/sme/src/";
    // ---.FSTs:
    //public static final String inverted_FST = " /Users/"+user_name+"/main/langs/sme/src/generator-dict-gt-norm.xfst";
    public static final String inverted_FST = " /Users/car010/main/langs/sme/src/generator-dict-gt-norm.hfstol"; // new fst
    //public static final String an_FST = " /Users/"+user_name+"/main/langs/sme/src/analyser-disamb-gt-desc.xfst";
    public static final String an_FST = " /Users/car010/main/langs/sme/src/analyser-disamb-gt-desc.hfstol"; // new fst
    // ---.cg3:
    public static String vislcg3_DisGrammarLoc = "/Users/"+user_name+"/main/langs/sme/src/syntax/disambiguator.cg3";
    public static String vislcg3_SyntGrammarLoc = "/Users/"+user_name+"/main/giella-shared/smi/src/syntax/konteaksta.cg3";

    //No need to change anything here: the definitions above are used + flags var + input/output file paths (saved in output/ folder)
    //public static final String lookup_Flags = "-flags mbTT -utf8";
    public static final String lookup_Flags = ""; // for new fst

    public static final String tools_Dir = "/Users/"+user_name+"/main/gt/script/";
    public static final String abbr_file = " --abbr="+abbr_Dir+"abbr.txt | --corr="+abbr_Dir+"corr.txt | ";
    public static final String preprocess_Pipeline = preprocess_Loc + " --abbr="+abbr_Dir+"abbr.txt | "+lookup_Loc+" "+lookup_Flags+" "+an_FST+" | "+lookup2cg_loc+" | ";
    public static final String lookup_2cgLoc = " | /opt/local/bin/perl " + lookup2cg_loc + " | ";

    public static String inputfile_Loc = "/Users/"+user_name+"/main/apps/teaksta/sme/output/cg3input";
    public static String outputfile_Loc = "/Users/"+user_name+"/main/apps/teaksta/sme/output/cg3output";
    public static String cg3GeneratorInputFile_Loc = "/Users/"+user_name+"/main/apps/teaksta/sme/output/cg3GeneratorInput.tmp";
    public static String cg3GeneratorOutputFile_Loc = "/Users/"+user_name+"/main/apps/teaksta/sme/output/cg3GeneratorOutput.tmp";
  */

  //GTOAHPA PATHS:
    //public static final String lookup_Loc = "/usr/bin/lookup"; //gtoahpa
    public static final String lookup_Loc = "/usr/local/bin/lookup"; //gtoahpa-01
    //public static final String lookup_Loc = "/usr/bin/hfst-optimized-lookup"; // for new fst
    //public static final String lookup_Flags = "-flags mbTT -utf8";
    public static final String lookup_Flags = ""; // for new fst

    //public static final String inverted_FST = " /opt/smi/sme/bin/generator-oahpa-gt-norm-dial_GG.xfst"; // opt/smi/sme/bin on gtlab;  or " /opt/smi/sme/bin/isme-GG.restr.fst"
    //public static final String inverted_FST = " /opt/smi/sme/bin/generator-dict-gt-norm.hfstol"; // new fst
    public static final String inverted_FST = " /opt/smi/sme/bin/generator-dict-gt-norm.xfst"; // new fst
    //public static final String an_FST = " /opt/smi/sme/bin/sme.fst";
    //public static final String an_FST = " /opt/smi/sme/bin/analyser-disamb-gt-desc.hfstol"; // new fst
    public static final String an_FST = " /opt/smi/sme/bin/analyser-disamb-gt-desc.xfst"; // new fst

    public static final String tools_Dir = "/opt/smi/sme/bin/";
    public static final String abbr_Dir = "/opt/smi/sme/bin/";
    public static final String abbr_file = " --abbr=/opt/smi/sme/bin/abbr.txt | --corr=/opt/smi/sme/bin/corr.txt | ";

    public static final String preprocess_Pipeline = "/home/heli/main/gt/script/preprocess"; // not sure if this is correct, anyway it seems the var is not in use
    public static final String preprocess_Loc = "/opt/smi/sme/bin/preprocess";
    public static final String lookup_2cgLoc = " | /opt/smi/sme/bin/lookup2cg | ";

    public static String inputfile_Loc = "/home/teaksta/output/cg3input";
    public static String outputfile_Loc = "/home/teaksta/output/cg3output";

    public static String cg3GeneratorInputFile_Loc = "/home/teaksta/output/cg3GeneratorInput.tmp";
    public static String cg3GeneratorOutputFile_Loc = "/home/teaksta/output/cg3GeneratorOutput.tmp";

    //public static String vislcg3_Loc = "/usr/local/bin/vislcg3"; //gtoahpa
    public static String vislcg3_Loc = "/bin/vislcg3"; //gtoahpa-01
    public static String vislcg3_DisGrammarLoc = "/opt/smi/sme/bin/disambiguator.cg3"; // before it was disambiguation.cg3
    public static String vislcg3_SyntGrammarLoc = "/opt/smi/sme/bin/konteaksta.cg3";
}
