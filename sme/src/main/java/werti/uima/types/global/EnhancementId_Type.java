
/* First created by JCasGen Tue Mar 08 10:25:10 CET 2022 */
package werti.uima.types.global;

import org.apache.uima.jcas.JCas;
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.cas.impl.TypeImpl;
import org.apache.uima.cas.Type;
import org.apache.uima.cas.impl.FeatureImpl;
import org.apache.uima.cas.Feature;
import org.apache.uima.jcas.tcas.DocumentAnnotation_Type;

/** The enhancement ID of this CAS.
 * Updated by JCasGen Tue Mar 08 10:25:10 CET 2022
 * @generated */
public class EnhancementId_Type extends DocumentAnnotation_Type {
  /** @generated */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = EnhancementId.typeIndexID;
  /** @generated 
     @modifiable */
  @SuppressWarnings ("hiding")
  public final static boolean featOkTst = JCasRegistry.getFeatOkTst("werti.uima.types.global.EnhancementId");
 
  /** @generated */
  final Feature casFeat_enhId;
  /** @generated */
  final int     casFeatCode_enhId;
  /** @generated
   * @param addr low level Feature Structure reference
   * @return the feature value 
   */ 
  public long getEnhId(int addr) {
        if (featOkTst && casFeat_enhId == null)
      jcas.throwFeatMissing("enhId", "werti.uima.types.global.EnhancementId");
    return ll_cas.ll_getLongValue(addr, casFeatCode_enhId);
  }
  /** @generated
   * @param addr low level Feature Structure reference
   * @param v value to set 
   */    
  public void setEnhId(int addr, long v) {
        if (featOkTst && casFeat_enhId == null)
      jcas.throwFeatMissing("enhId", "werti.uima.types.global.EnhancementId");
    ll_cas.ll_setLongValue(addr, casFeatCode_enhId, v);}
    
  



  /** initialize variables to correspond with Cas Type and Features
	 * @generated
	 * @param jcas JCas
	 * @param casType Type 
	 */
  public EnhancementId_Type(JCas jcas, Type casType) {
    super(jcas, casType);
    casImpl.getFSClassRegistry().addGeneratorForType((TypeImpl)this.casType, getFSGenerator());

 
    casFeat_enhId = jcas.getRequiredFeatureDE(casType, "enhId", "uima.cas.Long", featOkTst);
    casFeatCode_enhId  = (null == casFeat_enhId) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_enhId).getCode();

  }
}



    