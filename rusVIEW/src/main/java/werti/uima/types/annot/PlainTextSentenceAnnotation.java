

/* First created by JCasGen Thu Oct 16 21:28:30 CEST 2014 */
package werti.uima.types.annot;

import org.apache.uima.jcas.JCas; 
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.jcas.cas.TOP_Type;

import org.apache.uima.jcas.tcas.Annotation;


/** Sentence detection based on plain text input.
 * Updated by JCasGen Thu Oct 16 21:28:30 CEST 2014
 * XML source: /Users/rre014/lingsvn/EduardVIEW/desc/vislcg3TypeSystem.xml
 * @generated */
public class PlainTextSentenceAnnotation extends Annotation {
  /** @generated
   * @ordered 
   */
  public final static int typeIndexID = JCasRegistry.register(PlainTextSentenceAnnotation.class);
  /** @generated
   * @ordered 
   */
  public final static int type = typeIndexID;
  /** @generated  */
  public              int getTypeIndexID() {return typeIndexID;}
 
  /** Never called.  Disable default constructor
   * @generated */
  protected PlainTextSentenceAnnotation() {}
    
  /** Internal - constructor used by generator 
   * @generated */
  public PlainTextSentenceAnnotation(int addr, TOP_Type type) {
    super(addr, type);
    readObject();
  }
  
  /** @generated */
  public PlainTextSentenceAnnotation(JCas jcas) {
    super(jcas);
    readObject();   
  } 

  /** @generated */  
  public PlainTextSentenceAnnotation(JCas jcas, int begin, int end) {
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
     
}

    