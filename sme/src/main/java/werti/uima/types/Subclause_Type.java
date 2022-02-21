
/* First created by JCasGen Mon Feb 21 10:32:01 CET 2022 */
package werti.uima.types;

import org.apache.uima.jcas.JCas;
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.cas.impl.TypeImpl;
import org.apache.uima.cas.Type;
import org.apache.uima.cas.impl.FeatureImpl;
import org.apache.uima.cas.Feature;
import org.apache.uima.jcas.tcas.Annotation_Type;

/** 
 * Updated by JCasGen Mon Feb 21 10:32:01 CET 2022
 * @generated */
public class Subclause_Type extends Annotation_Type {
  /** @generated */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = Subclause.typeIndexID;
  /** @generated 
     @modifiable */
  @SuppressWarnings ("hiding")
  public final static boolean featOkTst = JCasRegistry.getFeatOkTst("werti.uima.types.Subclause");
 
  /** @generated */
  final Feature casFeat_modifiedSurface;
  /** @generated */
  final int     casFeatCode_modifiedSurface;
  /** @generated
   * @param addr low level Feature Structure reference
   * @return the feature value 
   */ 
  public String getModifiedSurface(int addr) {
        if (featOkTst && casFeat_modifiedSurface == null)
      jcas.throwFeatMissing("modifiedSurface", "werti.uima.types.Subclause");
    return ll_cas.ll_getStringValue(addr, casFeatCode_modifiedSurface);
  }
  /** @generated
   * @param addr low level Feature Structure reference
   * @param v value to set 
   */    
  public void setModifiedSurface(int addr, String v) {
        if (featOkTst && casFeat_modifiedSurface == null)
      jcas.throwFeatMissing("modifiedSurface", "werti.uima.types.Subclause");
    ll_cas.ll_setStringValue(addr, casFeatCode_modifiedSurface, v);}
    
  



  /** initialize variables to correspond with Cas Type and Features
	 * @generated
	 * @param jcas JCas
	 * @param casType Type 
	 */
  public Subclause_Type(JCas jcas, Type casType) {
    super(jcas, casType);
    casImpl.getFSClassRegistry().addGeneratorForType((TypeImpl)this.casType, getFSGenerator());

 
    casFeat_modifiedSurface = jcas.getRequiredFeatureDE(casType, "modifiedSurface", "uima.cas.String", featOkTst);
    casFeatCode_modifiedSurface  = (null == casFeat_modifiedSurface) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_modifiedSurface).getCode();

  }
}



    