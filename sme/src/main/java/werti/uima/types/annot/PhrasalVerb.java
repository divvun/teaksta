

/* First created by JCasGen Tue Mar 08 10:25:10 CET 2022 */
package werti.uima.types.annot;

import org.apache.uima.jcas.JCas; 
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.jcas.cas.TOP_Type;

import org.apache.uima.jcas.cas.FSArray;
import org.apache.uima.jcas.tcas.Annotation;


/** Annotations for phrasal verbs.
 * Updated by JCasGen Tue Mar 08 10:25:10 CET 2022
 * XML source: /home/boerre/repos/langtech/apps/teaksta/sme/desc/vislcg3TypeSystem.xml
 * @generated */
public class PhrasalVerb extends Annotation {
  /** @generated
   * @ordered 
   */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = JCasRegistry.register(PhrasalVerb.class);
  /** @generated
   * @ordered 
   */
  @SuppressWarnings ("hiding")
  public final static int type = typeIndexID;
  /** @generated
   * @return index of the type  
   */
  @Override
  public              int getTypeIndexID() {return typeIndexID;}
 
  /** Never called.  Disable default constructor
   * @generated */
  protected PhrasalVerb() {/* intentionally empty block */}
    
  /** Internal - constructor used by generator 
   * @generated
   * @param addr low level Feature Structure reference
   * @param type the type of this Feature Structure 
   */
  public PhrasalVerb(int addr, TOP_Type type) {
    super(addr, type);
    readObject();
  }
  
  /** @generated
   * @param jcas JCas to which this Feature Structure belongs 
   */
  public PhrasalVerb(JCas jcas) {
    super(jcas);
    readObject();   
  } 

  /** @generated
   * @param jcas JCas to which this Feature Structure belongs
   * @param begin offset to the begin spot in the SofA
   * @param end offset to the end spot in the SofA 
  */  
  public PhrasalVerb(JCas jcas, int begin, int end) {
    super(jcas);
    setBegin(begin);
    setEnd(end);
    readObject();
  }   

  /** 
   * <!-- begin-user-doc -->
   * Write your own initialization here
   * <!-- end-user-doc -->
   *
   * @generated modifiable 
   */
  private void readObject() {/*default - does nothing empty block */}
     
 
    
  //*--------------*
  //* Feature: verb

  /** getter for verb - gets The verb part of the phrasal verb.
   * @generated
   * @return value of the feature 
   */
  public FSArray getVerb() {
    if (PhrasalVerb_Type.featOkTst && ((PhrasalVerb_Type)jcasType).casFeat_verb == null)
      jcasType.jcas.throwFeatMissing("verb", "werti.uima.types.annot.PhrasalVerb");
    return (FSArray)(jcasType.ll_cas.ll_getFSForRef(jcasType.ll_cas.ll_getRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_verb)));}
    
  /** setter for verb - sets The verb part of the phrasal verb. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setVerb(FSArray v) {
    if (PhrasalVerb_Type.featOkTst && ((PhrasalVerb_Type)jcasType).casFeat_verb == null)
      jcasType.jcas.throwFeatMissing("verb", "werti.uima.types.annot.PhrasalVerb");
    jcasType.ll_cas.ll_setRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_verb, jcasType.ll_cas.ll_getFSRef(v));}    
    
  /** indexed getter for verb - gets an indexed value - The verb part of the phrasal verb.
   * @generated
   * @param i index in the array to get
   * @return value of the element at index i 
   */
  public Token getVerb(int i) {
    if (PhrasalVerb_Type.featOkTst && ((PhrasalVerb_Type)jcasType).casFeat_verb == null)
      jcasType.jcas.throwFeatMissing("verb", "werti.uima.types.annot.PhrasalVerb");
    jcasType.jcas.checkArrayBounds(jcasType.ll_cas.ll_getRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_verb), i);
    return (Token)(jcasType.ll_cas.ll_getFSForRef(jcasType.ll_cas.ll_getRefArrayValue(jcasType.ll_cas.ll_getRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_verb), i)));}

  /** indexed setter for verb - sets an indexed value - The verb part of the phrasal verb.
   * @generated
   * @param i index in the array to set
   * @param v value to set into the array 
   */
  public void setVerb(int i, Token v) { 
    if (PhrasalVerb_Type.featOkTst && ((PhrasalVerb_Type)jcasType).casFeat_verb == null)
      jcasType.jcas.throwFeatMissing("verb", "werti.uima.types.annot.PhrasalVerb");
    jcasType.jcas.checkArrayBounds(jcasType.ll_cas.ll_getRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_verb), i);
    jcasType.ll_cas.ll_setRefArrayValue(jcasType.ll_cas.ll_getRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_verb), i, jcasType.ll_cas.ll_getFSRef(v));}
   
    
  //*--------------*
  //* Feature: particle

  /** getter for particle - gets The particle part of the phrasal verb.
   * @generated
   * @return value of the feature 
   */
  public FSArray getParticle() {
    if (PhrasalVerb_Type.featOkTst && ((PhrasalVerb_Type)jcasType).casFeat_particle == null)
      jcasType.jcas.throwFeatMissing("particle", "werti.uima.types.annot.PhrasalVerb");
    return (FSArray)(jcasType.ll_cas.ll_getFSForRef(jcasType.ll_cas.ll_getRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_particle)));}
    
  /** setter for particle - sets The particle part of the phrasal verb. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setParticle(FSArray v) {
    if (PhrasalVerb_Type.featOkTst && ((PhrasalVerb_Type)jcasType).casFeat_particle == null)
      jcasType.jcas.throwFeatMissing("particle", "werti.uima.types.annot.PhrasalVerb");
    jcasType.ll_cas.ll_setRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_particle, jcasType.ll_cas.ll_getFSRef(v));}    
    
  /** indexed getter for particle - gets an indexed value - The particle part of the phrasal verb.
   * @generated
   * @param i index in the array to get
   * @return value of the element at index i 
   */
  public Token getParticle(int i) {
    if (PhrasalVerb_Type.featOkTst && ((PhrasalVerb_Type)jcasType).casFeat_particle == null)
      jcasType.jcas.throwFeatMissing("particle", "werti.uima.types.annot.PhrasalVerb");
    jcasType.jcas.checkArrayBounds(jcasType.ll_cas.ll_getRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_particle), i);
    return (Token)(jcasType.ll_cas.ll_getFSForRef(jcasType.ll_cas.ll_getRefArrayValue(jcasType.ll_cas.ll_getRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_particle), i)));}

  /** indexed setter for particle - sets an indexed value - The particle part of the phrasal verb.
   * @generated
   * @param i index in the array to set
   * @param v value to set into the array 
   */
  public void setParticle(int i, Token v) { 
    if (PhrasalVerb_Type.featOkTst && ((PhrasalVerb_Type)jcasType).casFeat_particle == null)
      jcasType.jcas.throwFeatMissing("particle", "werti.uima.types.annot.PhrasalVerb");
    jcasType.jcas.checkArrayBounds(jcasType.ll_cas.ll_getRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_particle), i);
    jcasType.ll_cas.ll_setRefArrayValue(jcasType.ll_cas.ll_getRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_particle), i, jcasType.ll_cas.ll_getFSRef(v));}
   
    
  //*--------------*
  //* Feature: np

  /** getter for np - gets The associated NP part for some phrasal verbs.
   * @generated
   * @return value of the feature 
   */
  public FSArray getNp() {
    if (PhrasalVerb_Type.featOkTst && ((PhrasalVerb_Type)jcasType).casFeat_np == null)
      jcasType.jcas.throwFeatMissing("np", "werti.uima.types.annot.PhrasalVerb");
    return (FSArray)(jcasType.ll_cas.ll_getFSForRef(jcasType.ll_cas.ll_getRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_np)));}
    
  /** setter for np - sets The associated NP part for some phrasal verbs. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setNp(FSArray v) {
    if (PhrasalVerb_Type.featOkTst && ((PhrasalVerb_Type)jcasType).casFeat_np == null)
      jcasType.jcas.throwFeatMissing("np", "werti.uima.types.annot.PhrasalVerb");
    jcasType.ll_cas.ll_setRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_np, jcasType.ll_cas.ll_getFSRef(v));}    
    
  /** indexed getter for np - gets an indexed value - The associated NP part for some phrasal verbs.
   * @generated
   * @param i index in the array to get
   * @return value of the element at index i 
   */
  public Token getNp(int i) {
    if (PhrasalVerb_Type.featOkTst && ((PhrasalVerb_Type)jcasType).casFeat_np == null)
      jcasType.jcas.throwFeatMissing("np", "werti.uima.types.annot.PhrasalVerb");
    jcasType.jcas.checkArrayBounds(jcasType.ll_cas.ll_getRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_np), i);
    return (Token)(jcasType.ll_cas.ll_getFSForRef(jcasType.ll_cas.ll_getRefArrayValue(jcasType.ll_cas.ll_getRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_np), i)));}

  /** indexed setter for np - sets an indexed value - The associated NP part for some phrasal verbs.
   * @generated
   * @param i index in the array to set
   * @param v value to set into the array 
   */
  public void setNp(int i, Token v) { 
    if (PhrasalVerb_Type.featOkTst && ((PhrasalVerb_Type)jcasType).casFeat_np == null)
      jcasType.jcas.throwFeatMissing("np", "werti.uima.types.annot.PhrasalVerb");
    jcasType.jcas.checkArrayBounds(jcasType.ll_cas.ll_getRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_np), i);
    jcasType.ll_cas.ll_setRefArrayValue(jcasType.ll_cas.ll_getRefValue(addr, ((PhrasalVerb_Type)jcasType).casFeatCode_np), i, jcasType.ll_cas.ll_getFSRef(v));}
  }

    