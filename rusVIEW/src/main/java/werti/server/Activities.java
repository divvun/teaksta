package werti.server;

import java.io.File;
import java.io.IOException;
import java.util.Iterator;
import java.util.TreeMap;

/**
 * Find activity specifications for all active activities.
 * 
 * @author Niels Ott?
 * @author Adriane Boyd
 *
 */
public class Activities implements Iterable<String> {

	public static final String ATT_NAME = "werti.activities";
	
	private TreeMap<String, ActivityConfiguration> configMap;
    
	public Activities(File actDir) throws IOException {
		configMap = new TreeMap<String, ActivityConfiguration>();
		
		for (File f : actDir.listFiles()) {
			if (f.isDirectory()) {
				ActivityConfiguration ac = new ActivityConfiguration(
						new File(f.getAbsolutePath() + File.separator + "activity.xml"));
				if (ac.isEnabled()) {
					configMap.put(f.getName(), ac);
				}
			}
		}
	}

	@Override
	public Iterator<String> iterator() {
		return configMap.keySet().iterator();
	}
	
	public ActivityConfiguration getActivity(String key) {
		return configMap.get(key);
	}
	
}
