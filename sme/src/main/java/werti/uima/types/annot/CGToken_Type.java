
/* First created by JCasGen Mon Feb 21 10:32:01 CET 2022 */
package werti.uima.types.annot;

import org.apache.uima.jcas.JCas;
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.cas.impl.TypeImpl;
import org.apache.uima.cas.Type;
import org.apache.uima.cas.impl.FeatureImpl;
import org.apache.uima.cas.Feature;

/** A token with added Constraint Grammar analysis information.
 * Updated by JCasGen Mon Feb 21 10:32:01 CET 2022
 * @generated */
public class CGToken_Type extends Token_Type {
  /** @generated */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = CGToken.typeIndexID;
  /** @generated 
     @modifiable */
  @SuppressWarnings ("hiding")
  public final static boolean featOkTst = JCasRegistry.getFeatOkTst("werti.uima.types.annot.CGToken");
 
  /** @generated */
  final Feature casFeat_readings;
  /** @generated */
  final int     casFeatCode_readings;
  /** @generated
   * @param addr low level Feature Structure reference
   * @return the feature value 
   */ 
  public int getReadings(int addr) {
        if (featOkTst && casFeat_readings == null)
      jcas.throwFeatMissing("readings", "werti.uima.types.annot.CGToken");
    return ll_cas.ll_getRefValue(addr, casFeatCode_readings);
  }
  /** @generated
   * @param addr low level Feature Structure reference
   * @param v value to set 
   */    
  public void setReadings(int addr, int v) {
        if (featOkTst && casFeat_readings == null)
      jcas.throwFeatMissing("readings", "werti.uima.types.annot.CGToken");
    ll_cas.ll_setRefValue(addr, casFeatCode_readings, v);}
    
   /** @generated
   * @param addr low level Feature Structure reference
   * @param i index of item in the array
   * @return value at index i in the array 
   */
  public int getReadings(int addr, int i) {
        if (featOkTst && casFeat_readings == null)
      jcas.throwFeatMissing("readings", "werti.uima.types.annot.CGToken");
    if (lowLevelTypeChecks)
      return ll_cas.ll_getRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_readings), i, true);
    jcas.checkArrayBounds(ll_cas.ll_getRefValue(addr, casFeatCode_readings), i);
	return ll_cas.ll_getRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_readings), i);
  }
   
  /** @generated
   * @param addr low level Feature Structure reference
   * @param i index of item in the array
   * @param v value to set
   */ 
  public void setReadings(int addr, int i, int v) {
        if (featOkTst && casFeat_readings == null)
      jcas.throwFeatMissing("readings", "werti.uima.types.annot.CGToken");
    if (lowLevelTypeChecks)
      ll_cas.ll_setRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_readings), i, v, true);
    jcas.checkArrayBounds(ll_cas.ll_getRefValue(addr, casFeatCode_readings), i);
    ll_cas.ll_setRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_readings), i, v);
  }
 



  /** initialize variables to correspond with Cas Type and Features
	 * @generated
	 * @param jcas JCas
	 * @param casType Type 
	 */
  public CGToken_Type(JCas jcas, Type casType) {
    super(jcas, casType);
    casImpl.getFSClassRegistry().addGeneratorForType((TypeImpl)this.casType, getFSGenerator());

 
    casFeat_readings = jcas.getRequiredFeatureDE(casType, "readings", "uima.cas.FSArray", featOkTst);
    casFeatCode_readings  = (null == casFeat_readings) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_readings).getCode();

  }
}



    