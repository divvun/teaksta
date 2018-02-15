package werti.util;

//Class containing all paths.
//To work locally uncomment (and change user name) the 'LOCAL PATHS' section and comment the 'GTOAHPA PATHS' section.

public final class Constants {
  //LOCAL PATHS:
  /*
    public static final String lookup_Loc = "/usr/local/bin/lookup";
    public static final String lookup_Flags = "-flags mbTT -utf8";

    public static final String inverted_FST = " /Users/car010/main/langs/sme/src/generator-gt-norm.xfst";
    public static final String an_FST = " /Users/car010/main/langs/sme/src/analyser-gt-norm.xfst";

    public static final String tools_Dir = "/Users/car010/main/gt/script/";
    public static final String abbr_Dir = "/Users/car010/main/langs/sme/src/";
    public static final String abbr_file = " --abbr=/Users/car010/main/langs/sme/src/abbr.txt | --corr=/Users/car010/main/langs/sme/src/corr.txt | ";

    public static final String preprocess_Pipeline = "/Users/car010/main/gt/script/preprocess --abbr=/Users/car010/main/langs/sme/src/abbr.txt | /usr/local/bin/lookup -flags mbTT -utf8 /Users/car010/main/langs/sme/src/analyser-gt-desc.xfst | /Users/car010/main/gt/script/lookup2cg | ";
  	public static final String preprocess_Loc = "/Users/car010/main/gt/script/preprocess";
  	public static final String lookup_2cgLoc = " | /opt/local/bin/perl /Users/car010/main/gt/script/lookup2cg | ";

    public static String inputfile_Loc = "/Users/car010/main/apps/teaksta/sme/output/cg3input";
    public static String outputfile_Loc = "/Users/car010/main/apps/teaksta/sme/output/cg3output";
  */

  //GTOAHPA PATHS:
    public final String lookup_Loc = "/usr/bin/lookup";
    public final String lookup_Flags = "-flags mbTT -utf8";

    public final String inverted_FST = " /opt/smi/sme/bin/isme-GG.restr.fst"; // opt/smi/sme/bin on gtlab;
    public final String an_FST = " /opt/smi/sme/bin/sme.fst";

    public static final String tools_Dir = "/opt/smi/sme/bin/";
    public static final String abbr_Dir = "/opt/smi/sme/bin/";
    public final String abbr_file = " --abbr=/opt/smi/sme/bin/abbr.txt | --corr=/opt/smi/sme/bin//corr.txt | ";

    public final String preprocess_Pipeline = "/home/heli/main/gt/script/preprocess"; // not sure if this is correct, anyway it seems the var is not in use
  	public final String preprocess_Loc = "/opt/smi/sme/bin/preprocess";
  	public final String lookup_2cgLoc = " | /opt/smi/sme/bin/lookup2cg | ";

    public String inputfile_Loc = "/home/teaksta/output/cg3input";
    public String outputfile_Loc = "/home/teaksta/output/cg3output";
}
