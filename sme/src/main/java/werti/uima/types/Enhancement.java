

/* First created by JCasGen Tue Mar 08 10:25:10 CET 2022 */
package werti.uima.types;

import org.apache.uima.jcas.JCas; 
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.jcas.cas.TOP_Type;

import org.apache.uima.jcas.tcas.Annotation;


/** Describes an enhancment on the current spot.
 * Updated by JCasGen Tue Mar 08 10:25:10 CET 2022
 * XML source: /home/boerre/repos/langtech/apps/teaksta/sme/desc/vislcg3TypeSystem.xml
 * @generated */
public class Enhancement extends Annotation {
  /** @generated
   * @ordered 
   */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = JCasRegistry.register(Enhancement.class);
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
  protected Enhancement() {/* intentionally empty block */}
    
  /** Internal - constructor used by generator 
   * @generated
   * @param addr low level Feature Structure reference
   * @param type the type of this Feature Structure 
   */
  public Enhancement(int addr, TOP_Type type) {
    super(addr, type);
    readObject();
  }
  
  /** @generated
   * @param jcas JCas to which this Feature Structure belongs 
   */
  public Enhancement(JCas jcas) {
    super(jcas);
    readObject();   
  } 

  /** @generated
   * @param jcas JCas to which this Feature Structure belongs
   * @param begin offset to the begin spot in the SofA
   * @param end offset to the end spot in the SofA 
  */  
  public Enhancement(JCas jcas, int begin, int end) {
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
  //* Feature: EnhanceStart

  /** getter for EnhanceStart - gets The start tag of the enhancement annotation.
   * @generated
   * @return value of the feature 
   */
  public String getEnhanceStart() {
    if (Enhancement_Type.featOkTst && ((Enhancement_Type)jcasType).casFeat_EnhanceStart == null)
      jcasType.jcas.throwFeatMissing("EnhanceStart", "werti.uima.types.Enhancement");
    return jcasType.ll_cas.ll_getStringValue(addr, ((Enhancement_Type)jcasType).casFeatCode_EnhanceStart);}
    
  /** setter for EnhanceStart - sets The start tag of the enhancement annotation. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setEnhanceStart(String v) {
    if (Enhancement_Type.featOkTst && ((Enhancement_Type)jcasType).casFeat_EnhanceStart == null)
      jcasType.jcas.throwFeatMissing("EnhanceStart", "werti.uima.types.Enhancement");
    jcasType.ll_cas.ll_setStringValue(addr, ((Enhancement_Type)jcasType).casFeatCode_EnhanceStart, v);}    
   
    
  //*--------------*
  //* Feature: EnhanceEnd

  /** getter for EnhanceEnd - gets The end tag of the enhancement annotation.
   * @generated
   * @return value of the feature 
   */
  public String getEnhanceEnd() {
    if (Enhancement_Type.featOkTst && ((Enhancement_Type)jcasType).casFeat_EnhanceEnd == null)
      jcasType.jcas.throwFeatMissing("EnhanceEnd", "werti.uima.types.Enhancement");
    return jcasType.ll_cas.ll_getStringValue(addr, ((Enhancement_Type)jcasType).casFeatCode_EnhanceEnd);}
    
  /** setter for EnhanceEnd - sets The end tag of the enhancement annotation. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setEnhanceEnd(String v) {
    if (Enhancement_Type.featOkTst && ((Enhancement_Type)jcasType).casFeat_EnhanceEnd == null)
      jcasType.jcas.throwFeatMissing("EnhanceEnd", "werti.uima.types.Enhancement");
    jcasType.ll_cas.ll_setStringValue(addr, ((Enhancement_Type)jcasType).casFeatCode_EnhanceEnd, v);}    
   
    
  //*--------------*
  //* Feature: Relevant

  /** getter for Relevant - gets Whether this annotation will be relevant for the activity.
   * @generated
   * @return value of the feature 
   */
  public boolean getRelevant() {
    if (Enhancement_Type.featOkTst && ((Enhancement_Type)jcasType).casFeat_Relevant == null)
      jcasType.jcas.throwFeatMissing("Relevant", "werti.uima.types.Enhancement");
    return jcasType.ll_cas.ll_getBooleanValue(addr, ((Enhancement_Type)jcasType).casFeatCode_Relevant);}
    
  /** setter for Relevant - sets Whether this annotation will be relevant for the activity. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setRelevant(boolean v) {
    if (Enhancement_Type.featOkTst && ((Enhancement_Type)jcasType).casFeat_Relevant == null)
      jcasType.jcas.throwFeatMissing("Relevant", "werti.uima.types.Enhancement");
    jcasType.ll_cas.ll_setBooleanValue(addr, ((Enhancement_Type)jcasType).casFeatCode_Relevant, v);}    
  }

    