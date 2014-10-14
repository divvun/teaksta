package werti.uima.ae.filter;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Set;
import java.util.TreeSet;

import werti.uima.types.annot.Token;

/**
 * The PassiveSentenceFilter decides whether it is worthwhile to parse a 
 * sentence in anticipation of performing an active/passive transformation.
 * 
 * @author Adriane Boyd
 *
 */
public class PassiveSentenceFilter implements Filter {
	/*
	 * This list is from simple English wiktionary.
	 * 
	 * http://simple.wiktionary.org/w/index.php?title=Category:Transitive_verbs
	 * 
	 * It is noisy and inaccurate, but provides a starting point
	 * I'd imagine it should be replaced with verbs from FrameNet or the
	 * Levin verb book, maybe from here:
	 * 
	 * http://www-personal.umich.edu/~jlawler/levin.verbs
	 */
	private Set<String> transitiveVerbs = new TreeSet<String>(Arrays.asList(
			"abandon", "absorb", "abstract", "abuse", "accept", "access", 
			"accommodate", "accompany", "ache for", "achieve", "acknowledge", "act", 
			"adapt", "add", "address", "adjust", "admit", "adopt", "advance", 
			"advertise", "advise", "advocate", "affect", "afford", "affright", 
			"aggregate", "aim", "alarm", "allocate", "ally", "alter", "amaze", 
			"ambuscade", "amplify", "analyse", "analyze", "anticipate", "appall", 
			"appeal", "appease", "apply", "appoint", "appreciate", "approach", 
			"appropriate", "approve", "approximate", "arrange", "ascend", "ascribe", 
			"ashame", "assemble", "assess", "assign", "associate", "assume", "assure", 
			"attach", "attack", "attempt", "attract", "attribute", "authorise", 
			"authorize", "avoid", "awake", "award", "axe", "back", "background", 
			"badge", "bake", "balance", "ball", "ban", "bare", "bargain", "base", 
			"batter", "battle", "beam", "bear", "beat", "become", "beg", "begin", 
			"believe", "bench", "bend", "best", "bet", "bias", "bid", "bill", "bind", 
			"bite", "black", "blame", "blast", "blaze", "bless", "blind", "block", 
			"blow", "board", "bog", "boil", "bomb", "bond", "book", "bore", "borrow", 
			"boss", "bother", "bottle", "bow", "bowl", "box", "bracket", "breach", 
			"break", "break down", "breast", "brief", "bring", "broom", "budget", 
			"bug", "bugger", "build", "bully", "bump", "bunch", "buoy", "burden", 
			"burgle", "burst", "bury", "bus", "busy", "butcher", "buy", "cake", 
			"calculate", "call", "calm", "can", "cancel", "cap", "captain", "capture", 
			"card", "carpet", "carry", "cash", "cast", "catch", "cater", "cause", 
			"cease", "celebrate", "center", "centre", "challenge", "chamber", 
			"channel", "charge", "chart", "chase", "cheat", "check", "cheer", "cheer up", 
			"chip", "choke", "choose", "chop", "circle", "claim", "clean", 
			"click", "climb", "clobber", "close", "clothe", "club", "clue", "clutch", 
			"coach", "cock", "code", "coin", "collar", "collect", "combine", 
			"comfort", "command", "commence", "comment", "commission", "commit", 
			"communicate", "compact", "compare", "compensate", "complement", 
			"complete", "complicate", "compound", "compress", "comprise", "compute", 
			"conceive", "concern", "conduct", "confer", "confine", "confuse", 
			"congregate", "connect", "consider", "construct", "consult", "consume", 
			"contact", "contain", "content", "continue", "contract", "contrast", 
			"contribute", "control", "convert", "convey", "cook", "cool", "copyright", 
			"core", "corrupt", "cost", "count", "couple", "court", "crane", "crash", 
			"create", "credit", "cripple", "crisp", "crook", "crop", "crowd", "cure", 
			"cycle", "dam", "damage", "damn", "dance", "dash", "date", "deal", 
			"debate", "debut", "decide", "declare", "decline", "decorate", "decrease", 
			"dedicate", "defeat", "delay", "delegate", "delight", "deliver", "demand", 
			"demonize", "demonstrate", "demoralise", "demoralize", "denote", "deny", 
			"derive", "descend", "describe", "design", "desolate", "detect", 
			"develop", "devote", "diagram", "dice", "dig", "dim", "diminish", 
			"direct", "disable", "disappoint", "discipline", "discover", "discuss", 
			"disguise", "disjoin", "disperse", "dissolve", "distort", "distract", 
			"disturb", "disunite", "ditch", "divide", "divorce", "dock", "doctor", 
			"document", "dog", "dominate", "dot", "double", "doubt", "draft", "drag", 
			"dress", "drink", "drive", "driving", "drug", "dry", "duct tape", "dump", 
			"dust", "dye", "ease", "eat", "elbow", "eliminate", "embarrass", 
			"emphasize", "employ", "enable", "encounter", "encourage", "endanger", 
			"engage", "engineer", "enjoy", "ensure", "enter", "entitle", "envision", 
			"equal", "equip", "eschew", "essay", "establish", "estimate", "evaluate", 
			"evolve", "ex", "exact", "examine", "exceed", "except", "excite", 
			"excuse", "exercise", "expand", "expect", "expel", "expend", "explain", 
			"explode", "explore", "extend", "extract", "facilitate", "familiarise", 
			"familiarize", "fan", "fancy", "fashion", "fault", "fear", "feature", 
			"feed", "feel", "fetch", "field", "figure", "file", "fill", "film", 
			"filter", "finance", "find", "fine", "finger", "finish", "fire", "fit", 
			"fix", "flame", "flap", "flatter", "flavor", "flavour", "fly", "focus", 
			"fog", "follow", "fool", "foot", "forbid", "forget", "format", "forward", 
			"fox", "free", "freeze", "frighten", "fringe", "fry", "fuck", "fumble", 
			"fund", "gangbang", "gas", "gather", "generate", "google", "govern", 
			"grab", "graduate", "greet", "ground", "grow", "grudge", "guarantee", 
			"guaranty", "guard", "guess", "hail", "halt", "harm", "harvest", "hate", 
			"hawk", "head", "hearing", "hide", "highlight", "hit", "hold", "hop", 
			"host", "house", "hunt", "hurry", "hurt", "identify", "ignite", "ignore", 
			"illuminate", "illustrate", "imagine", "imply", "impose", "impress", 
			"improve", "include", "index", "indicate", "induce", "industrialise", 
			"industrialize", "infer", "inform", "initiate", "input", "insert", 
			"inspect", "install", "institute", "instruct", "insult", "insure", 
			"intend", "interpret", "interrupt", "introduce", "invade", "invest", 
			"investigate", "invite", "involve", "irritate", "issue", "jack", "jacket", 
			"jerk", "jest", "join", "jolly", "juice", "jump", "justify", "kick", 
			"know", "label", "labor", "labour", "lack", "land", "lay", "lease", 
			"leave", "lend", "libel", "light", "limit", "link", "list", "load", 
			"lose", "machine", "mail", "maintain", "manage", "maneuver", "manoeuvre", 
			"market", "master", "match", "mate", "matriculate", "mean", "measure", 
			"meet", "mention", "meow", "merge", "microwave", "midwife", "milk", 
			"mine", "minimise", "minimize", "mirror", "misbehave", "mistake", 
			"misunderstand", "model", "moderate", "modify", "mold", "mop", "mortgage", 
			"mother", "motivate", "mould", "move", "muck", "mug", "multiply", "name", 
			"napalm", "narrow", "needle", "neglect", "negotiate", "nominate", 
			"notice", "notify", "nurse", "oblige", "obscure", "observe", "obtain", 
			"occasion", "occupy", "offset", "ok", "okay", "omen", "opaque", "operate", 
			"oppose", "oppress", "order", "organise", "organize", "output", "oxidize", 
			"pack", "pad", "page", "pair", "paragraph", "park", "part", "partner", 
			"pass", "pat", "patch", "pattern", "pay", "people", "perceive", "perfect", 
			"persuade", "phase", "photo", "photograph", "pick", "picture", "piece", 
			"pile", "pimp", "place", "plan", "plane", "pledge", "plug", "poison", 
			"polish", "poll", "portion", "position", "post", "pound", "powder", 
			"power", "practice", "pray", "precede", "predicate", "predict", "prefer", 
			"prepare", "present", "press", "presume", "prevent", "prime", "print", 
			"prize", "process", "produce", "profit", "program", "programme", 
			"project", "promote", "propose", "prospect", "prove", "provide", 
			"publish", "punctuate", "punish", "purchase", "purge", "pursue", "push", 
			"puzzle", "qualify", "quarter", "radio", "raise", "reach", "realise", 
			"realize", "rear", "reason", "recall", "receive", "reckon", "recommend", 
			"recover", "recruit", "reduce", "refer", "referred", "reflect", 
			"refrigerate", "refuse", "regard", "register", "regularize", "regulate", 
			"reject", "relate", "relax", "remark", "remedy", "remember", "remind", 
			"remove", "rent", "repair", "repeat", "repel", "replace", "reply", 
			"report", "research", "reserve", "resist", "restore", "restrict", 
			"restructure", "retain", "retire", "return", "reveal", "revenge", 
			"reverse", "revise", "revolt", "reward", "rid", "ridicule", "rival", 
			"rub", "rubbish", "rumor", "rush", "salt", "sample", "saw", "scapegoat", 
			"scare", "schedule", "scold", "scope", "scream", "season", "seat", 
			"section", "see", "seek", "select", "sentence", "serve", "set", "settle", 
			"shackle", "shadow", "shape", "share", "shave", "shed", "shift", "shine", 
			"shit", "shore", "shout", "show", "shower", "shylock", "sicken", "signal", 
			"sing", "sink", "sit", "site", "size", "sketch", "slather", "slide", 
			"slow", "smash", "smoke", "snake", "soap", "sock", "sod", "solve", 
			"sought", "space", "spare", "spawn", "specify", "spend", "spit", "split", 
			"sponsor", "spot", "staff", "stake out", "stall", "state", "stimulate", 
			"stock", "stole", "stomach", "store", "strengthen", "stress", "strike", 
			"strip", "stripe", "study", "subject", "subjugate", "submit", 
			"subordinate", "sue", "supply", "suppress", "survey", "survive", 
			"suspect", "suspend", "sustain", "swallow", "swap", "swarm", "swing", 
			"switch", "tackle", "tag", "tail", "take", "take up", "tape", "taste", 
			"tax", "teach", "team", "tell", "tend", "term", "terminate", "test", 
			"thin", "think", "throw", "throw away", "thump", "ticket", "tickle", 
			"tidy", "tie", "tile", "time", "tip", "total", "touch", "tough", "tour", 
			"trace", "track", "trail", "train", "transfer", "translate", "transmit", 
			"trap", "treasure", "treat", "trust", "try", "tune", "turn", "tutor", 
			"type", "undergo", "underlie", "undertake", "undo", "unify", "unlock", 
			"update", "urge", "vacuum", "vary", "vassal", "venerate", "vent", 
			"violate", "visualize", "vote", "wage", "wake up", "wank", "want", "ward", 
			"water", "waterlog", "weaken", "wear", "weigh", "wet", "whale", "will", 
			"win", "wire", "withdraw", "witness", "wonder", "wound", "wreck", 
			"wrestle", "write", "wrong", "yell"
	));
	
	public PassiveSentenceFilter() {
		// does this need to to anything?
	}
	
	/**
	 * 
	 * @param tokenlist	list of tokens from cas
	 * @return whether this sentence should be parsed for use in the activity
	 */
	@Override
	public boolean filter(List<Token> tokenlist) {
		// if the sentence is too long, skip
		if (tokenlist.size() > 20) {
			return false;
		}

		List<String> tokens = new ArrayList<String>();
		List<String> tags = new ArrayList<String>();
		//List<String> chunks = new ArrayList<String>();
		List<String> lemmas = new ArrayList<String>();
	
		for(Token t : tokenlist) {
			tokens.add(t.getCoveredText());
			tags.add(t.getTag());
			//chunks.add(t.getChunk());
			lemmas.add(t.getLemma());
		}
		
		// if the sentence doesn't contain any transitive verbs, skip
		boolean transitiveFound = false;
		for (int i = 0; i < tokens.size(); i++) {
			if (tags.get(i).matches("^V.*$") && transitiveVerbs.contains(lemmas.get(i))) {
				transitiveFound = true;
				break;
			}
		}		
		if (!transitiveFound) {
			return false;
		}
		
		// passed all filters
		return true;
	}
}
