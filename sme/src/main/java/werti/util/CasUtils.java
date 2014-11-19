package werti.util;

import java.util.Iterator;

import org.apache.uima.cas.text.AnnotationIndex;
import org.apache.uima.jcas.JCas;

import werti.uima.types.global.EnhancementId;

/**
 * Utility methods for CAS's.
 * @author Marion Zepf
 */
public class CasUtils {
	
	private static Iterator<EnhancementId> getEnhIdIterator(JCas cas) {
		AnnotationIndex enhIdIndex = cas.getAnnotationIndex(EnhancementId.type);
		Iterator<EnhancementId> enhIdIter = enhIdIndex.iterator();
		return enhIdIter;
	}
	
	/**
	 * the CAS is valid iff (all of) its enhancement ID(s) is not negative. 
	 * If the CAS has no enhancement ID, it is considered valid. But if the
	 * CAS is null, it's not valid.
	 * @param cas
	 * @return true iff the CAS is empty
	 */
	public static boolean isValid(JCas cas) {
		if (cas == null) {
			return false;
		}
		Iterator<EnhancementId> enhIdIter = getEnhIdIterator(cas);
		while (enhIdIter.hasNext()) {
			EnhancementId enhIdFS = enhIdIter.next();
			long enhId = enhIdFS.getEnhId();
			if (enhId < 0) {
				return false;
			}
		}
		return true;
	}
	
	/**
	 * make the CAS invalid by setting its enhancement ID(s) to -1.
	 * @param cas
	 */
	public static void makeInvalid(JCas cas) {
		Iterator<EnhancementId> enhIdIter = getEnhIdIterator(cas);
		while (enhIdIter.hasNext()) {
			EnhancementId enhIdFS = enhIdIter.next();
			enhIdFS.setEnhId(-1);
		}
	}
	
	/**
	 * adds a new enhancement ID annotation to the CAS. If the CAS already 
	 * has an enhancement ID, it will NOT be overridden. Instead, the CAS 
	 * would have two enhancement IDs then.
	 * @param cas
	 * @param enhId
	 */
	public static void addEnhId(JCas cas, long enhId) {
		EnhancementId enhIdFS = new EnhancementId(cas);
		enhIdFS.setEnhId(enhId);
		enhIdFS.addToIndexes();
	}
	
	/**
	 * has the reset() method been called on this CAS? A good indicator for 
	 * this is whether the document language is empty.
	 * @param cas
	 * @return true iff the CAS has been reset
	 */
	public static boolean hasBeenReset(JCas cas) {
		return cas.getDocumentLanguage() == null;
	}
}


