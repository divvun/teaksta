
/* First created by JCasGen Tue Mar 08 10:25:10 CET 2022 */
package werti.uima.types;

import org.apache.uima.jcas.JCas;
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.cas.impl.TypeImpl;
import org.apache.uima.cas.Type;
import org.apache.uima.cas.impl.FeatureImpl;
import org.apache.uima.cas.Feature;
import org.apache.uima.jcas.tcas.Annotation_Type;

/** Describes an enhancment on the current spot.
 * Updated by JCasGen Tue Mar 08 10:25:10 CET 2022
 * @generated */
public class Enhancement_Type extends Annotation_Type {
  /** @generated */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = Enhancement.typeIndexID;
  /** @generated 
     @modifiable */
  @SuppressWarnings ("hiding")
  public final static boolean featOkTst = JCasRegistry.getFeatOkTst("werti.uima.types.Enhancement");
 
  /** @generated */
  final Feature casFeat_EnhanceStart;
  /** @generated */
  final int     casFeatCode_EnhanceStart;
  /** @generated
   * @param addr low level Feature Structure reference
   * @return the feature value 
   */ 
  public String getEnhanceStart(int addr) {
        if (featOkTst && casFeat_EnhanceStart == null)
      jcas.throwFeatMissing("EnhanceStart", "werti.uima.types.Enhancement");
    return ll_cas.ll_getStringValue(addr, casFeatCode_EnhanceStart);
  }
  /** @generated
   * @param addr low level Feature Structure reference
   * @param v value to set 
   */    
  public void setEnhanceStart(int addr, String v) {
        if (featOkTst && casFeat_EnhanceStart == null)
      jcas.throwFeatMissing("EnhanceStart", "werti.uima.types.Enhancement");
    ll_cas.ll_setStringValue(addr, casFeatCode_EnhanceStart, v);}
    
  
 
  /** @generated */
  final Feature casFeat_EnhanceEnd;
  /** @generated */
  final int     casFeatCode_EnhanceEnd;
  /** @generated
   * @param addr low level Feature Structure reference
   * @return the feature value 
   */ 
  public String getEnhanceEnd(int addr) {
        if (featOkTst && casFeat_EnhanceEnd == null)
      jcas.throwFeatMissing("EnhanceEnd", "werti.uima.types.Enhancement");
    return ll_cas.ll_getStringValue(addr, casFeatCode_EnhanceEnd);
  }
  /** @generated
   * @param addr low level Feature Structure reference
   * @param v value to set 
   */    
  public void setEnhanceEnd(int addr, String v) {
        if (featOkTst && casFeat_EnhanceEnd == null)
      jcas.throwFeatMissing("EnhanceEnd", "werti.uima.types.Enhancement");
    ll_cas.ll_setStringValue(addr, casFeatCode_EnhanceEnd, v);}
    
  
 
  /** @generated */
  final Feature casFeat_Relevant;
  /** @generated */
  final int     casFeatCode_Relevant;
  /** @generated
   * @param addr low level Feature Structure reference
   * @return the feature value 
   */ 
  public boolean getRelevant(int addr) {
        if (featOkTst && casFeat_Relevant == null)
      jcas.throwFeatMissing("Relevant", "werti.uima.types.Enhancement");
    return ll_cas.ll_getBooleanValue(addr, casFeatCode_Relevant);
  }
  /** @generated
   * @param addr low level Feature Structure reference
   * @param v value to set 
   */    
  public void setRelevant(int addr, boolean v) {
        if (featOkTst && casFeat_Relevant == null)
      jcas.throwFeatMissing("Relevant", "werti.uima.types.Enhancement");
    ll_cas.ll_setBooleanValue(addr, casFeatCode_Relevant, v);}
    
  



  /** initialize variables to correspond with Cas Type and Features
	 * @generated
	 * @param jcas JCas
	 * @param casType Type 
	 */
  public Enhancement_Type(JCas jcas, Type casType) {
    super(jcas, casType);
    casImpl.getFSClassRegistry().addGeneratorForType((TypeImpl)this.casType, getFSGenerator());

 
    casFeat_EnhanceStart = jcas.getRequiredFeatureDE(casType, "EnhanceStart", "uima.cas.String", featOkTst);
    casFeatCode_EnhanceStart  = (null == casFeat_EnhanceStart) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_EnhanceStart).getCode();

 
    casFeat_EnhanceEnd = jcas.getRequiredFeatureDE(casType, "EnhanceEnd", "uima.cas.String", featOkTst);
    casFeatCode_EnhanceEnd  = (null == casFeat_EnhanceEnd) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_EnhanceEnd).getCode();

 
    casFeat_Relevant = jcas.getRequiredFeatureDE(casType, "Relevant", "uima.cas.Boolean", featOkTst);
    casFeatCode_Relevant  = (null == casFeat_Relevant) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_Relevant).getCode();

  }
}



    