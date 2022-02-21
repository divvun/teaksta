
/* First created by JCasGen Mon Feb 21 10:32:01 CET 2022 */
package werti.uima.types.annot;

import org.apache.uima.jcas.JCas;
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.cas.impl.TypeImpl;
import org.apache.uima.cas.Type;
import org.apache.uima.jcas.cas.NonEmptyStringList_Type;

/** A reading in a constraint grammar cohort.
 * Updated by JCasGen Mon Feb 21 10:32:01 CET 2022
 * @generated */
public class CGReading_Type extends NonEmptyStringList_Type {
  /** @generated */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = CGReading.typeIndexID;
  /** @generated 
     @modifiable */
  @SuppressWarnings ("hiding")
  public final static boolean featOkTst = JCasRegistry.getFeatOkTst("werti.uima.types.annot.CGReading");



  /** initialize variables to correspond with Cas Type and Features
	 * @generated
	 * @param jcas JCas
	 * @param casType Type 
	 */
  public CGReading_Type(JCas jcas, Type casType) {
    super(jcas, casType);
    casImpl.getFSClassRegistry().addGeneratorForType((TypeImpl)this.casType, getFSGenerator());

  }
}



    