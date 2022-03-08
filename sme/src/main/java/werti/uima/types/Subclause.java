

/* First created by JCasGen Tue Mar 08 10:25:10 CET 2022 */
package werti.uima.types;

import org.apache.uima.jcas.JCas; 
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.jcas.cas.TOP_Type;

import org.apache.uima.jcas.tcas.Annotation;


/** 
 * Updated by JCasGen Tue Mar 08 10:25:10 CET 2022
 * XML source: /home/boerre/repos/langtech/apps/teaksta/sme/desc/vislcg3TypeSystem.xml
 * @generated */
public class Subclause extends Annotation {
  /** @generated
   * @ordered 
   */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = JCasRegistry.register(Subclause.class);
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
  protected Subclause() {/* intentionally empty block */}
    
  /** Internal - constructor used by generator 
   * @generated
   * @param addr low level Feature Structure reference
   * @param type the type of this Feature Structure 
   */
  public Subclause(int addr, TOP_Type type) {
    super(addr, type);
    readObject();
  }
  
  /** @generated
   * @param jcas JCas to which this Feature Structure belongs 
   */
  public Subclause(JCas jcas) {
    super(jcas);
    readObject();   
  } 

  /** @generated
   * @param jcas JCas to which this Feature Structure belongs
   * @param begin offset to the begin spot in the SofA
   * @param end offset to the end spot in the SofA 
  */  
  public Subclause(JCas jcas, int begin, int end) {
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
  //* Feature: modifiedSurface

  /** getter for modifiedSurface - gets 
   * @generated
   * @return value of the feature 
   */
  public String getModifiedSurface() {
    if (Subclause_Type.featOkTst && ((Subclause_Type)jcasType).casFeat_modifiedSurface == null)
      jcasType.jcas.throwFeatMissing("modifiedSurface", "werti.uima.types.Subclause");
    return jcasType.ll_cas.ll_getStringValue(addr, ((Subclause_Type)jcasType).casFeatCode_modifiedSurface);}
    
  /** setter for modifiedSurface - sets  
   * @generated
   * @param v value to set into the feature 
   */
  public void setModifiedSurface(String v) {
    if (Subclause_Type.featOkTst && ((Subclause_Type)jcasType).casFeat_modifiedSurface == null)
      jcasType.jcas.throwFeatMissing("modifiedSurface", "werti.uima.types.Subclause");
    jcasType.ll_cas.ll_setStringValue(addr, ((Subclause_Type)jcasType).casFeatCode_modifiedSurface, v);}    
  }

    