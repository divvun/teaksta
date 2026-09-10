package werti.server;

import java.io.File;
import java.io.IOException;
import java.util.HashSet;
import java.util.Iterator;
import java.util.Set;
import java.util.TreeMap;

/**
 * Find activity specifications for all active activities.
 * 
 * @author Niels Ott?
 * @author Adriane Boyd
 *
 */
// [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities+1]
public class Activities implements Iterable<String> {

	public static final String ATT_NAME = "werti.activities";
	
	private TreeMap<String, ActivityConfiguration> configMap;
	private Set<String> ignoredActivities;
    
	// [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities.activities-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.activities-fn]
	public Activities(File actDir) throws IOException {
		configMap = new TreeMap<String, ActivityConfiguration>();
		
		ignoredActivities = new HashSet<String>();
		ignoredActivities.add("Conditionals");
		
		for (File f : actDir.listFiles()) {
			if (f.isDirectory() && !ignoredActivities.contains(f.getName())) {
				configMap.put(f.getName(), new ActivityConfiguration(
						new File(f.getAbsolutePath() + File.separator + "activity.xml")));
			}
		}
	}

	// [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities.iterator-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.iterator-fn]
	public Iterator<String> iterator() {
		return configMap.keySet().iterator();
	}
	
	// [spec:teaksta:def:sme.src.main.java.werti.server.activities.activities.get-activity-fn]
	// [spec:teaksta:sem:sme.src.main.java.werti.server.activities.activities.get-activity-fn]
	public ActivityConfiguration getActivity(String key) {
		return configMap.get(key);
	}
	
}
