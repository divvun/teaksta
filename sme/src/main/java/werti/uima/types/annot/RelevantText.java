

/* First created by JCasGen Tue Mar 08 10:25:10 CET 2022 */
package werti.uima.types.annot;

import org.apache.uima.jcas.JCas; 
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.jcas.cas.TOP_Type;

import org.apache.uima.jcas.tcas.Annotation;


/** Optional annotation to specify which text to work on.
 * Updated by JCasGen Tue Mar 08 10:25:10 CET 2022
 * XML source: /home/boerre/repos/langtech/apps/teaksta/sme/desc/vislcg3TypeSystem.xml
 * @generated */
public class RelevantText extends Annotation {
  /** @generated
   * @ordered 
   */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = JCasRegistry.register(RelevantText.class);
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
  protected RelevantText() {/* intentionally empty block */}
    
  /** Internal - constructor used by generator 
   * @generated
   * @param addr low level Feature Structure reference
   * @param type the type of this Feature Structure 
   */
  public RelevantText(int addr, TOP_Type type) {
    super(addr, type);
    readObject();
  }
  
  /** @generated
   * @param jcas JCas to which this Feature Structure belongs 
   */
  public RelevantText(JCas jcas) {
    super(jcas);
    readObject();   
  } 

  /** @generated
   * @param jcas JCas to which this Feature Structure belongs
   * @param begin offset to the begin spot in the SofA
   * @param end offset to the end spot in the SofA 
  */  
  public RelevantText(JCas jcas, int begin, int end) {
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
  //* Feature: htmlContentType

  /** getter for htmlContentType - gets The text type classification (headline, main, comments, etc.).
   * @generated
   * @return value of the feature 
   */
  public String getHtmlContentType() {
    if (RelevantText_Type.featOkTst && ((RelevantText_Type)jcasType).casFeat_htmlContentType == null)
      jcasType.jcas.throwFeatMissing("htmlContentType", "werti.uima.types.annot.RelevantText");
    return jcasType.ll_cas.ll_getStringValue(addr, ((RelevantText_Type)jcasType).casFeatCode_htmlContentType);}
    
  /** setter for htmlContentType - sets The text type classification (headline, main, comments, etc.). 
   * @generated
   * @param v value to set into the feature 
   */
  public void setHtmlContentType(String v) {
    if (RelevantText_Type.featOkTst && ((RelevantText_Type)jcasType).casFeat_htmlContentType == null)
      jcasType.jcas.throwFeatMissing("htmlContentType", "werti.uima.types.annot.RelevantText");
    jcasType.ll_cas.ll_setStringValue(addr, ((RelevantText_Type)jcasType).casFeatCode_htmlContentType, v);}    
   
    
  //*--------------*
  //* Feature: enclosing_tag

  /** getter for enclosing_tag - gets The html tag that encloses this text fragment.
   * @generated
   * @return value of the feature 
   */
  public String getEnclosing_tag() {
    if (RelevantText_Type.featOkTst && ((RelevantText_Type)jcasType).casFeat_enclosing_tag == null)
      jcasType.jcas.throwFeatMissing("enclosing_tag", "werti.uima.types.annot.RelevantText");
    return jcasType.ll_cas.ll_getStringValue(addr, ((RelevantText_Type)jcasType).casFeatCode_enclosing_tag);}
    
  /** setter for enclosing_tag - sets The html tag that encloses this text fragment. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setEnclosing_tag(String v) {
    if (RelevantText_Type.featOkTst && ((RelevantText_Type)jcasType).casFeat_enclosing_tag == null)
      jcasType.jcas.throwFeatMissing("enclosing_tag", "werti.uima.types.annot.RelevantText");
    jcasType.ll_cas.ll_setStringValue(addr, ((RelevantText_Type)jcasType).casFeatCode_enclosing_tag, v);}    
   
    
  //*--------------*
  //* Feature: relevant

  /** getter for relevant - gets Is this a relevant chunk of input or too small?
            The idea is that a piece of text can make it to being relevant, iff it is between to already relevant pieces of text, but itself too small to be included and all tags that were opened between the first piece of actually relevant text and this piece of irrelevant text are closed before the second piece starts.
            The rationale behind this is not confusing the tagger by randomly dropping words, just because they're inside some <b> tag.
   * @generated
   * @return value of the feature 
   */
  public boolean getRelevant() {
    if (RelevantText_Type.featOkTst && ((RelevantText_Type)jcasType).casFeat_relevant == null)
      jcasType.jcas.throwFeatMissing("relevant", "werti.uima.types.annot.RelevantText");
    return jcasType.ll_cas.ll_getBooleanValue(addr, ((RelevantText_Type)jcasType).casFeatCode_relevant);}
    
  /** setter for relevant - sets Is this a relevant chunk of input or too small?
            The idea is that a piece of text can make it to being relevant, iff it is between to already relevant pieces of text, but itself too small to be included and all tags that were opened between the first piece of actually relevant text and this piece of irrelevant text are closed before the second piece starts.
            The rationale behind this is not confusing the tagger by randomly dropping words, just because they're inside some <b> tag. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setRelevant(boolean v) {
    if (RelevantText_Type.featOkTst && ((RelevantText_Type)jcasType).casFeat_relevant == null)
      jcasType.jcas.throwFeatMissing("relevant", "werti.uima.types.annot.RelevantText");
    jcasType.ll_cas.ll_setBooleanValue(addr, ((RelevantText_Type)jcasType).casFeatCode_relevant, v);}    
  }

    