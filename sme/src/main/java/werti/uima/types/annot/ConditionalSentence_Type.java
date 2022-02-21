
/* First created by JCasGen Mon Feb 21 10:32:01 CET 2022 */
package werti.uima.types.annot;

import org.apache.uima.jcas.JCas;
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.cas.impl.TypeImpl;
import org.apache.uima.cas.Type;
import org.apache.uima.cas.impl.FeatureImpl;
import org.apache.uima.cas.Feature;

/** A sentence containing a conditional.
 * Updated by JCasGen Mon Feb 21 10:32:01 CET 2022
 * @generated */
public class ConditionalSentence_Type extends SentenceAnnotation_Type {
  /** @generated */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = ConditionalSentence.typeIndexID;
  /** @generated 
     @modifiable */
  @SuppressWarnings ("hiding")
  public final static boolean featOkTst = JCasRegistry.getFeatOkTst("werti.uima.types.annot.ConditionalSentence");
 
  /** @generated */
  final Feature casFeat_trigger;
  /** @generated */
  final int     casFeatCode_trigger;
  /** @generated
   * @param addr low level Feature Structure reference
   * @return the feature value 
   */ 
  public int getTrigger(int addr) {
        if (featOkTst && casFeat_trigger == null)
      jcas.throwFeatMissing("trigger", "werti.uima.types.annot.ConditionalSentence");
    return ll_cas.ll_getRefValue(addr, casFeatCode_trigger);
  }
  /** @generated
   * @param addr low level Feature Structure reference
   * @param v value to set 
   */    
  public void setTrigger(int addr, int v) {
        if (featOkTst && casFeat_trigger == null)
      jcas.throwFeatMissing("trigger", "werti.uima.types.annot.ConditionalSentence");
    ll_cas.ll_setRefValue(addr, casFeatCode_trigger, v);}
    
   /** @generated
   * @param addr low level Feature Structure reference
   * @param i index of item in the array
   * @return value at index i in the array 
   */
  public int getTrigger(int addr, int i) {
        if (featOkTst && casFeat_trigger == null)
      jcas.throwFeatMissing("trigger", "werti.uima.types.annot.ConditionalSentence");
    if (lowLevelTypeChecks)
      return ll_cas.ll_getRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_trigger), i, true);
    jcas.checkArrayBounds(ll_cas.ll_getRefValue(addr, casFeatCode_trigger), i);
	return ll_cas.ll_getRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_trigger), i);
  }
   
  /** @generated
   * @param addr low level Feature Structure reference
   * @param i index of item in the array
   * @param v value to set
   */ 
  public void setTrigger(int addr, int i, int v) {
        if (featOkTst && casFeat_trigger == null)
      jcas.throwFeatMissing("trigger", "werti.uima.types.annot.ConditionalSentence");
    if (lowLevelTypeChecks)
      ll_cas.ll_setRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_trigger), i, v, true);
    jcas.checkArrayBounds(ll_cas.ll_getRefValue(addr, casFeatCode_trigger), i);
    ll_cas.ll_setRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_trigger), i, v);
  }
 
 
  /** @generated */
  final Feature casFeat_condition;
  /** @generated */
  final int     casFeatCode_condition;
  /** @generated
   * @param addr low level Feature Structure reference
   * @return the feature value 
   */ 
  public int getCondition(int addr) {
        if (featOkTst && casFeat_condition == null)
      jcas.throwFeatMissing("condition", "werti.uima.types.annot.ConditionalSentence");
    return ll_cas.ll_getRefValue(addr, casFeatCode_condition);
  }
  /** @generated
   * @param addr low level Feature Structure reference
   * @param v value to set 
   */    
  public void setCondition(int addr, int v) {
        if (featOkTst && casFeat_condition == null)
      jcas.throwFeatMissing("condition", "werti.uima.types.annot.ConditionalSentence");
    ll_cas.ll_setRefValue(addr, casFeatCode_condition, v);}
    
   /** @generated
   * @param addr low level Feature Structure reference
   * @param i index of item in the array
   * @return value at index i in the array 
   */
  public int getCondition(int addr, int i) {
        if (featOkTst && casFeat_condition == null)
      jcas.throwFeatMissing("condition", "werti.uima.types.annot.ConditionalSentence");
    if (lowLevelTypeChecks)
      return ll_cas.ll_getRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_condition), i, true);
    jcas.checkArrayBounds(ll_cas.ll_getRefValue(addr, casFeatCode_condition), i);
	return ll_cas.ll_getRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_condition), i);
  }
   
  /** @generated
   * @param addr low level Feature Structure reference
   * @param i index of item in the array
   * @param v value to set
   */ 
  public void setCondition(int addr, int i, int v) {
        if (featOkTst && casFeat_condition == null)
      jcas.throwFeatMissing("condition", "werti.uima.types.annot.ConditionalSentence");
    if (lowLevelTypeChecks)
      ll_cas.ll_setRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_condition), i, v, true);
    jcas.checkArrayBounds(ll_cas.ll_getRefValue(addr, casFeatCode_condition), i);
    ll_cas.ll_setRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_condition), i, v);
  }
 
 
  /** @generated */
  final Feature casFeat_result;
  /** @generated */
  final int     casFeatCode_result;
  /** @generated
   * @param addr low level Feature Structure reference
   * @return the feature value 
   */ 
  public int getResult(int addr) {
        if (featOkTst && casFeat_result == null)
      jcas.throwFeatMissing("result", "werti.uima.types.annot.ConditionalSentence");
    return ll_cas.ll_getRefValue(addr, casFeatCode_result);
  }
  /** @generated
   * @param addr low level Feature Structure reference
   * @param v value to set 
   */    
  public void setResult(int addr, int v) {
        if (featOkTst && casFeat_result == null)
      jcas.throwFeatMissing("result", "werti.uima.types.annot.ConditionalSentence");
    ll_cas.ll_setRefValue(addr, casFeatCode_result, v);}
    
   /** @generated
   * @param addr low level Feature Structure reference
   * @param i index of item in the array
   * @return value at index i in the array 
   */
  public int getResult(int addr, int i) {
        if (featOkTst && casFeat_result == null)
      jcas.throwFeatMissing("result", "werti.uima.types.annot.ConditionalSentence");
    if (lowLevelTypeChecks)
      return ll_cas.ll_getRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_result), i, true);
    jcas.checkArrayBounds(ll_cas.ll_getRefValue(addr, casFeatCode_result), i);
	return ll_cas.ll_getRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_result), i);
  }
   
  /** @generated
   * @param addr low level Feature Structure reference
   * @param i index of item in the array
   * @param v value to set
   */ 
  public void setResult(int addr, int i, int v) {
        if (featOkTst && casFeat_result == null)
      jcas.throwFeatMissing("result", "werti.uima.types.annot.ConditionalSentence");
    if (lowLevelTypeChecks)
      ll_cas.ll_setRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_result), i, v, true);
    jcas.checkArrayBounds(ll_cas.ll_getRefValue(addr, casFeatCode_result), i);
    ll_cas.ll_setRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_result), i, v);
  }
 



  /** initialize variables to correspond with Cas Type and Features
	 * @generated
	 * @param jcas JCas
	 * @param casType Type 
	 */
  public ConditionalSentence_Type(JCas jcas, Type casType) {
    super(jcas, casType);
    casImpl.getFSClassRegistry().addGeneratorForType((TypeImpl)this.casType, getFSGenerator());

 
    casFeat_trigger = jcas.getRequiredFeatureDE(casType, "trigger", "uima.cas.FSArray", featOkTst);
    casFeatCode_trigger  = (null == casFeat_trigger) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_trigger).getCode();

 
    casFeat_condition = jcas.getRequiredFeatureDE(casType, "condition", "uima.cas.FSArray", featOkTst);
    casFeatCode_condition  = (null == casFeat_condition) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_condition).getCode();

 
    casFeat_result = jcas.getRequiredFeatureDE(casType, "result", "uima.cas.FSArray", featOkTst);
    casFeatCode_result  = (null == casFeat_result) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_result).getCode();

  }
}



    