

/* First created by JCasGen Wed May 07 20:43:31 CEST 2014 */
package werti.uima.types.annot;

import org.apache.uima.jcas.JCas; 
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.jcas.cas.TOP_Type;

import org.apache.uima.jcas.cas.NonEmptyStringList;


/** A reading in a constraint grammar cohort.
 * Updated by JCasGen Wed May 07 20:43:31 CEST 2014
 * XML source: /home/ruskonteaksta/desc/vislcg3TypeSystem.xml
 * @generated */
public class CGReading extends NonEmptyStringList {
  /** @generated
   * @ordered 
   */
  public final static int typeIndexID = JCasRegistry.register(CGReading.class);
  /** @generated
   * @ordered 
   */
  public final static int type = typeIndexID;
  /** @generated  */
  public              int getTypeIndexID() {return typeIndexID;}
 
  /** Never called.  Disable default constructor
   * @generated */
  protected CGReading() {}
    
  /** Internal - constructor used by generator 
   * @generated */
  public CGReading(int addr, TOP_Type type) {
    super(addr, type);
    readObject();
  }
  
  /** @generated */
  public CGReading(JCas jcas) {
    super(jcas);
    readObject();   
  } 

  /** <!-- begin-user-doc -->
    * Write your own initialization here
    * <!-- end-user-doc -->
  @generated modifiable */
  private void readObject() {}
     
}

    