//! Class containing all paths.
//!
//! The values below are the GTOAHPA (gtoahpa-01) deployment paths; the Java
//! source keeps a commented-out LOCAL PATHS section for working off-server,
//! switched by hand.
//!
//! Several of these are shell fragments rather than plain paths — note the
//! embedded pipes and the leading spaces, which the call sites concatenate
//! straight into a command line.

// [spec:teaksta:def:sme.src.main.java.werti.util.constants.constants]

pub const LOOKUP_LOC: &str = "/usr/local/bin/lookup";
pub const LOOKUP_FLAGS: &str = "";

pub const INVERTED_FST: &str = " /opt/smi/sme/bin/generator-dict-gt-norm.xfst";
pub const AN_FST: &str = " /opt/smi/sme/bin/analyser-disamb-gt-desc.xfst";

pub const TOOLS_DIR: &str = "/opt/smi/sme/bin/";
pub const ABBR_DIR: &str = "/opt/smi/sme/bin/";
pub const ABBR_FILE: &str =
    " --abbr=/opt/smi/sme/bin/abbr.txt | --corr=/opt/smi/sme/bin/corr.txt | ";

/// Not in use by any call site.
pub const PREPROCESS_PIPELINE: &str = "/home/heli/main/gt/script/preprocess";
pub const PREPROCESS_LOC: &str = "/opt/smi/sme/bin/preprocess";
pub const LOOKUP_2CG_LOC: &str = " | /opt/smi/sme/bin/lookup2cg | ";

pub const INPUTFILE_LOC: &str = "/home/teaksta/output/cg3input";
pub const OUTPUTFILE_LOC: &str = "/home/teaksta/output/cg3output";

pub const CG3_GENERATOR_INPUT_FILE_LOC: &str = "/home/teaksta/output/cg3GeneratorInput.tmp";
pub const CG3_GENERATOR_OUTPUT_FILE_LOC: &str = "/home/teaksta/output/cg3GeneratorOutput.tmp";

pub const VISLCG3_LOC: &str = "/bin/vislcg3";
/// Named disambiguation.cg3 in earlier deployments.
pub const VISLCG3_DIS_GRAMMAR_LOC: &str = "/opt/smi/sme/bin/disambiguator.cg3";
pub const VISLCG3_SYNT_GRAMMAR_LOC: &str = "/opt/smi/sme/bin/konteaksta.cg3";
