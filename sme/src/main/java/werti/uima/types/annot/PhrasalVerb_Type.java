
/* First created by JCasGen Tue Mar 08 10:25:10 CET 2022 */
package werti.uima.types.annot;

import org.apache.uima.jcas.JCas;
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.cas.impl.TypeImpl;
import org.apache.uima.cas.Type;
import org.apache.uima.cas.impl.FeatureImpl;
import org.apache.uima.cas.Feature;
import org.apache.uima.jcas.tcas.Annotation_Type;

/** Annotations for phrasal verbs.
 * Updated by JCasGen Tue Mar 08 10:25:10 CET 2022
 * @generated */
public class PhrasalVerb_Type extends Annotation_Type {
  /** @generated */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = PhrasalVerb.typeIndexID;
  /** @generated 
     @modifiable */
  @SuppressWarnings ("hiding")
  public final static boolean featOkTst = JCasRegistry.getFeatOkTst("werti.uima.types.annot.PhrasalVerb");
 
  /** @generated */
  final Feature casFeat_verb;
  /** @generated */
  final int     casFeatCode_verb;
  /** @generated
   * @param addr low level Feature Structure reference
   * @return the feature value 
   */ 
  public int getVerb(int addr) {
        if (featOkTst && casFeat_verb == null)
      jcas.throwFeatMissing("verb", "werti.uima.types.annot.PhrasalVerb");
    return ll_cas.ll_getRefValue(addr, casFeatCode_verb);
  }
  /** @generated
   * @param addr low level Feature Structure reference
   * @param v value to set 
   */    
  public void setVerb(int addr, int v) {
        if (featOkTst && casFeat_verb == null)
      jcas.throwFeatMissing("verb", "werti.uima.types.annot.PhrasalVerb");
    ll_cas.ll_setRefValue(addr, casFeatCode_verb, v);}
    
   /** @generated
   * @param addr low level Feature Structure reference
   * @param i index of item in the array
   * @return value at index i in the array 
   */
  public int getVerb(int addr, int i) {
        if (featOkTst && casFeat_verb == null)
      jcas.throwFeatMissing("verb", "werti.uima.types.annot.PhrasalVerb");
    if (lowLevelTypeChecks)
      return ll_cas.ll_getRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_verb), i, true);
    jcas.checkArrayBounds(ll_cas.ll_getRefValue(addr, casFeatCode_verb), i);
	return ll_cas.ll_getRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_verb), i);
  }
   
  /** @generated
   * @param addr low level Feature Structure reference
   * @param i index of item in the array
   * @param v value to set
   */ 
  public void setVerb(int addr, int i, int v) {
        if (featOkTst && casFeat_verb == null)
      jcas.throwFeatMissing("verb", "werti.uima.types.annot.PhrasalVerb");
    if (lowLevelTypeChecks)
      ll_cas.ll_setRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_verb), i, v, true);
    jcas.checkArrayBounds(ll_cas.ll_getRefValue(addr, casFeatCode_verb), i);
    ll_cas.ll_setRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_verb), i, v);
  }
 
 
  /** @generated */
  final Feature casFeat_particle;
  /** @generated */
  final int     casFeatCode_particle;
  /** @generated
   * @param addr low level Feature Structure reference
   * @return the feature value 
   */ 
  public int getParticle(int addr) {
        if (featOkTst && casFeat_particle == null)
      jcas.throwFeatMissing("particle", "werti.uima.types.annot.PhrasalVerb");
    return ll_cas.ll_getRefValue(addr, casFeatCode_particle);
  }
  /** @generated
   * @param addr low level Feature Structure reference
   * @param v value to set 
   */    
  public void setParticle(int addr, int v) {
        if (featOkTst && casFeat_particle == null)
      jcas.throwFeatMissing("particle", "werti.uima.types.annot.PhrasalVerb");
    ll_cas.ll_setRefValue(addr, casFeatCode_particle, v);}
    
   /** @generated
   * @param addr low level Feature Structure reference
   * @param i index of item in the array
   * @return value at index i in the array 
   */
  public int getParticle(int addr, int i) {
        if (featOkTst && casFeat_particle == null)
      jcas.throwFeatMissing("particle", "werti.uima.types.annot.PhrasalVerb");
    if (lowLevelTypeChecks)
      return ll_cas.ll_getRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_particle), i, true);
    jcas.checkArrayBounds(ll_cas.ll_getRefValue(addr, casFeatCode_particle), i);
	return ll_cas.ll_getRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_particle), i);
  }
   
  /** @generated
   * @param addr low level Feature Structure reference
   * @param i index of item in the array
   * @param v value to set
   */ 
  public void setParticle(int addr, int i, int v) {
        if (featOkTst && casFeat_particle == null)
      jcas.throwFeatMissing("particle", "werti.uima.types.annot.PhrasalVerb");
    if (lowLevelTypeChecks)
      ll_cas.ll_setRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_particle), i, v, true);
    jcas.checkArrayBounds(ll_cas.ll_getRefValue(addr, casFeatCode_particle), i);
    ll_cas.ll_setRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_particle), i, v);
  }
 
 
  /** @generated */
  final Feature casFeat_np;
  /** @generated */
  final int     casFeatCode_np;
  /** @generated
   * @param addr low level Feature Structure reference
   * @return the feature value 
   */ 
  public int getNp(int addr) {
        if (featOkTst && casFeat_np == null)
      jcas.throwFeatMissing("np", "werti.uima.types.annot.PhrasalVerb");
    return ll_cas.ll_getRefValue(addr, casFeatCode_np);
  }
  /** @generated
   * @param addr low level Feature Structure reference
   * @param v value to set 
   */    
  public void setNp(int addr, int v) {
        if (featOkTst && casFeat_np == null)
      jcas.throwFeatMissing("np", "werti.uima.types.annot.PhrasalVerb");
    ll_cas.ll_setRefValue(addr, casFeatCode_np, v);}
    
   /** @generated
   * @param addr low level Feature Structure reference
   * @param i index of item in the array
   * @return value at index i in the array 
   */
  public int getNp(int addr, int i) {
        if (featOkTst && casFeat_np == null)
      jcas.throwFeatMissing("np", "werti.uima.types.annot.PhrasalVerb");
    if (lowLevelTypeChecks)
      return ll_cas.ll_getRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_np), i, true);
    jcas.checkArrayBounds(ll_cas.ll_getRefValue(addr, casFeatCode_np), i);
	return ll_cas.ll_getRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_np), i);
  }
   
  /** @generated
   * @param addr low level Feature Structure reference
   * @param i index of item in the array
   * @param v value to set
   */ 
  public void setNp(int addr, int i, int v) {
        if (featOkTst && casFeat_np == null)
      jcas.throwFeatMissing("np", "werti.uima.types.annot.PhrasalVerb");
    if (lowLevelTypeChecks)
      ll_cas.ll_setRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_np), i, v, true);
    jcas.checkArrayBounds(ll_cas.ll_getRefValue(addr, casFeatCode_np), i);
    ll_cas.ll_setRefArrayValue(ll_cas.ll_getRefValue(addr, casFeatCode_np), i, v);
  }
 



  /** initialize variables to correspond with Cas Type and Features
	 * @generated
	 * @param jcas JCas
	 * @param casType Type 
	 */
  public PhrasalVerb_Type(JCas jcas, Type casType) {
    super(jcas, casType);
    casImpl.getFSClassRegistry().addGeneratorForType((TypeImpl)this.casType, getFSGenerator());

 
    casFeat_verb = jcas.getRequiredFeatureDE(casType, "verb", "uima.cas.FSArray", featOkTst);
    casFeatCode_verb  = (null == casFeat_verb) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_verb).getCode();

 
    casFeat_particle = jcas.getRequiredFeatureDE(casType, "particle", "uima.cas.FSArray", featOkTst);
    casFeatCode_particle  = (null == casFeat_particle) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_particle).getCode();

 
    casFeat_np = jcas.getRequiredFeatureDE(casType, "np", "uima.cas.FSArray", featOkTst);
    casFeatCode_np  = (null == casFeat_np) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_np).getCode();

  }
}



    