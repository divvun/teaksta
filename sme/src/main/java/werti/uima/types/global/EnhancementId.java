

/* First created by JCasGen Tue Mar 08 10:25:10 CET 2022 */
package werti.uima.types.global;

import org.apache.uima.jcas.JCas; 
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.jcas.cas.TOP_Type;

import org.apache.uima.jcas.tcas.DocumentAnnotation;


/** The enhancement ID of this CAS.
 * Updated by JCasGen Tue Mar 08 10:25:10 CET 2022
 * XML source: /home/boerre/repos/langtech/apps/teaksta/sme/desc/vislcg3TypeSystem.xml
 * @generated */
public class EnhancementId extends DocumentAnnotation {
  /** @generated
   * @ordered 
   */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = JCasRegistry.register(EnhancementId.class);
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
  protected EnhancementId() {/* intentionally empty block */}
    
  /** Internal - constructor used by generator 
   * @generated
   * @param addr low level Feature Structure reference
   * @param type the type of this Feature Structure 
   */
  public EnhancementId(int addr, TOP_Type type) {
    super(addr, type);
    readObject();
  }
  
  /** @generated
   * @param jcas JCas to which this Feature Structure belongs 
   */
  public EnhancementId(JCas jcas) {
    super(jcas);
    readObject();   
  } 

  /** @generated
   * @param jcas JCas to which this Feature Structure belongs
   * @param begin offset to the begin spot in the SofA
   * @param end offset to the end spot in the SofA 
  */  
  public EnhancementId(JCas jcas, int begin, int end) {
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
  //* Feature: enhId

  /** getter for enhId - gets 
   * @generated
   * @return value of the feature 
   */
  public long getEnhId() {
    if (EnhancementId_Type.featOkTst && ((EnhancementId_Type)jcasType).casFeat_enhId == null)
      jcasType.jcas.throwFeatMissing("enhId", "werti.uima.types.global.EnhancementId");
    return jcasType.ll_cas.ll_getLongValue(addr, ((EnhancementId_Type)jcasType).casFeatCode_enhId);}
    
  /** setter for enhId - sets  
   * @generated
   * @param v value to set into the feature 
   */
  public void setEnhId(long v) {
    if (EnhancementId_Type.featOkTst && ((EnhancementId_Type)jcasType).casFeat_enhId == null)
      jcasType.jcas.throwFeatMissing("enhId", "werti.uima.types.global.EnhancementId");
    jcasType.ll_cas.ll_setLongValue(addr, ((EnhancementId_Type)jcasType).casFeatCode_enhId, v);}    
  }

    