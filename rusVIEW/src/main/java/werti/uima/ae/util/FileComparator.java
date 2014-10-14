package werti.uima.ae.util;

import java.io.File;
import java.util.Comparator;

/**
 * Compares two files against each other using the 
 * last modified date.
 * @author Eduard
 *
 */

public class FileComparator implements Comparator<File>{

	@Override
	public int compare(File f1, File f2)
    {
        return Long.valueOf(f1.lastModified()).compareTo(f2.lastModified());
    }

}
