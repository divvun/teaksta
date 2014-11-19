

/* First created by JCasGen Tue Nov 18 13:56:47 CET 2014 */
package werti.uima.types.global;

import org.apache.uima.jcas.JCas; 
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.jcas.cas.TOP_Type;

import org.apache.uima.jcas.tcas.DocumentAnnotation;


/** The enhancement ID of this CAS.
 * Updated by JCasGen Tue Nov 18 13:56:47 CET 2014
 * XML source: /Users/mslm/main/apps/teaksta/sme/desc/vislcg3TypeSystem.xml
 * @generated */
public class EnhancementId extends DocumentAnnotation {
  /** @generated
   * @ordered 
   */
  public final static int typeIndexID = JCasRegistry.register(EnhancementId.class);
  /** @generated
   * @ordered 
   */
  public final static int type = typeIndexID;
  /** @generated  */
  public              int getTypeIndexID() {return typeIndexID;}
 
  /** Never called.  Disable default constructor
   * @generated */
  protected EnhancementId() {}
    
  /** Internal - constructor used by generator 
   * @generated */
  public EnhancementId(int addr, TOP_Type type) {
    super(addr, type);
    readObject();
  }
  
  /** @generated */
  public EnhancementId(JCas jcas) {
    super(jcas);
    readObject();   
  } 

  /** @generated */  
  public EnhancementId(JCas jcas, int begin, int end) {
    super(jcas);
    setBegin(begin);
    setEnd(end);
    readObject();
  }   

  /** <!-- begin-user-doc -->
    * Write your own initialization here
    * <!-- end-user-doc -->
  @generated modifiable */
  private void readObject() {}
     
 
    
  //*--------------*
  //* Feature: enhId

  /** getter for enhId - gets 
   * @generated */
  public long getEnhId() {
    if (EnhancementId_Type.featOkTst && ((EnhancementId_Type)jcasType).casFeat_enhId == null)
      jcasType.jcas.throwFeatMissing("enhId", "werti.uima.types.global.EnhancementId");
    return jcasType.ll_cas.ll_getLongValue(addr, ((EnhancementId_Type)jcasType).casFeatCode_enhId);}
    
  /** setter for enhId - sets  
   * @generated */
  public void setEnhId(long v) {
    if (EnhancementId_Type.featOkTst && ((EnhancementId_Type)jcasType).casFeat_enhId == null)
      jcasType.jcas.throwFeatMissing("enhId", "werti.uima.types.global.EnhancementId");
    jcasType.ll_cas.ll_setLongValue(addr, ((EnhancementId_Type)jcasType).casFeatCode_enhId, v);}    
  }

    