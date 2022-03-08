

/* First created by JCasGen Tue Mar 08 10:25:10 CET 2022 */
package werti.uima.types.annot;

import org.apache.uima.jcas.JCas; 
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.jcas.cas.TOP_Type;

import org.apache.uima.jcas.tcas.Annotation;


/** A relevant Token with PoS information attached.
 * Updated by JCasGen Tue Mar 08 10:25:10 CET 2022
 * XML source: /home/boerre/repos/langtech/apps/teaksta/sme/desc/vislcg3TypeSystem.xml
 * @generated */
public class Token extends Annotation {
  /** @generated
   * @ordered 
   */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = JCasRegistry.register(Token.class);
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
  protected Token() {/* intentionally empty block */}
    
  /** Internal - constructor used by generator 
   * @generated
   * @param addr low level Feature Structure reference
   * @param type the type of this Feature Structure 
   */
  public Token(int addr, TOP_Type type) {
    super(addr, type);
    readObject();
  }
  
  /** @generated
   * @param jcas JCas to which this Feature Structure belongs 
   */
  public Token(JCas jcas) {
    super(jcas);
    readObject();   
  } 

  /** @generated
   * @param jcas JCas to which this Feature Structure belongs
   * @param begin offset to the begin spot in the SofA
   * @param end offset to the end spot in the SofA 
  */  
  public Token(JCas jcas, int begin, int end) {
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
  //* Feature: tag

  /** getter for tag - gets Part of speech.
   * @generated
   * @return value of the feature 
   */
  public String getTag() {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_tag == null)
      jcasType.jcas.throwFeatMissing("tag", "werti.uima.types.annot.Token");
    return jcasType.ll_cas.ll_getStringValue(addr, ((Token_Type)jcasType).casFeatCode_tag);}
    
  /** setter for tag - sets Part of speech. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setTag(String v) {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_tag == null)
      jcasType.jcas.throwFeatMissing("tag", "werti.uima.types.annot.Token");
    jcasType.ll_cas.ll_setStringValue(addr, ((Token_Type)jcasType).casFeatCode_tag, v);}    
   
    
  //*--------------*
  //* Feature: detailedtag

  /** getter for detailedtag - gets Detailed part of speech.
   * @generated
   * @return value of the feature 
   */
  public String getDetailedtag() {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_detailedtag == null)
      jcasType.jcas.throwFeatMissing("detailedtag", "werti.uima.types.annot.Token");
    return jcasType.ll_cas.ll_getStringValue(addr, ((Token_Type)jcasType).casFeatCode_detailedtag);}
    
  /** setter for detailedtag - sets Detailed part of speech. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setDetailedtag(String v) {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_detailedtag == null)
      jcasType.jcas.throwFeatMissing("detailedtag", "werti.uima.types.annot.Token");
    jcasType.ll_cas.ll_setStringValue(addr, ((Token_Type)jcasType).casFeatCode_detailedtag, v);}    
   
    
  //*--------------*
  //* Feature: lemma

  /** getter for lemma - gets The lemma of the word in this token. May remain empty if there is no lemmatizer around.
   * @generated
   * @return value of the feature 
   */
  public String getLemma() {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_lemma == null)
      jcasType.jcas.throwFeatMissing("lemma", "werti.uima.types.annot.Token");
    return jcasType.ll_cas.ll_getStringValue(addr, ((Token_Type)jcasType).casFeatCode_lemma);}
    
  /** setter for lemma - sets The lemma of the word in this token. May remain empty if there is no lemmatizer around. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setLemma(String v) {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_lemma == null)
      jcasType.jcas.throwFeatMissing("lemma", "werti.uima.types.annot.Token");
    jcasType.ll_cas.ll_setStringValue(addr, ((Token_Type)jcasType).casFeatCode_lemma, v);}    
   
    
  //*--------------*
  //* Feature: gerund

  /** getter for gerund - gets The -ing form of the word in this token, if it is an infinitive. May remain empty if there is no generator around or if it's not an infinitive.
   * @generated
   * @return value of the feature 
   */
  public String getGerund() {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_gerund == null)
      jcasType.jcas.throwFeatMissing("gerund", "werti.uima.types.annot.Token");
    return jcasType.ll_cas.ll_getStringValue(addr, ((Token_Type)jcasType).casFeatCode_gerund);}
    
  /** setter for gerund - sets The -ing form of the word in this token, if it is an infinitive. May remain empty if there is no generator around or if it's not an infinitive. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setGerund(String v) {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_gerund == null)
      jcasType.jcas.throwFeatMissing("gerund", "werti.uima.types.annot.Token");
    jcasType.ll_cas.ll_setStringValue(addr, ((Token_Type)jcasType).casFeatCode_gerund, v);}    
   
    
  //*--------------*
  //* Feature: chunk

  /** getter for chunk - gets The chunk tags for this token.
   * @generated
   * @return value of the feature 
   */
  public String getChunk() {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_chunk == null)
      jcasType.jcas.throwFeatMissing("chunk", "werti.uima.types.annot.Token");
    return jcasType.ll_cas.ll_getStringValue(addr, ((Token_Type)jcasType).casFeatCode_chunk);}
    
  /** setter for chunk - sets The chunk tags for this token. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setChunk(String v) {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_chunk == null)
      jcasType.jcas.throwFeatMissing("chunk", "werti.uima.types.annot.Token");
    jcasType.ll_cas.ll_setStringValue(addr, ((Token_Type)jcasType).casFeatCode_chunk, v);}    
   
    
  //*--------------*
  //* Feature: mltag

  /** getter for mltag - gets A tag added by an activity-specific machine learning-based tagger.
   * @generated
   * @return value of the feature 
   */
  public String getMltag() {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_mltag == null)
      jcasType.jcas.throwFeatMissing("mltag", "werti.uima.types.annot.Token");
    return jcasType.ll_cas.ll_getStringValue(addr, ((Token_Type)jcasType).casFeatCode_mltag);}
    
  /** setter for mltag - sets A tag added by an activity-specific machine learning-based tagger. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setMltag(String v) {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_mltag == null)
      jcasType.jcas.throwFeatMissing("mltag", "werti.uima.types.annot.Token");
    jcasType.ll_cas.ll_setStringValue(addr, ((Token_Type)jcasType).casFeatCode_mltag, v);}    
   
    
  //*--------------*
  //* Feature: depid

  /** getter for depid - gets Token ID used to stored dependency parse (should we use position instead?).
   * @generated
   * @return value of the feature 
   */
  public int getDepid() {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_depid == null)
      jcasType.jcas.throwFeatMissing("depid", "werti.uima.types.annot.Token");
    return jcasType.ll_cas.ll_getIntValue(addr, ((Token_Type)jcasType).casFeatCode_depid);}
    
  /** setter for depid - sets Token ID used to stored dependency parse (should we use position instead?). 
   * @generated
   * @param v value to set into the feature 
   */
  public void setDepid(int v) {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_depid == null)
      jcasType.jcas.throwFeatMissing("depid", "werti.uima.types.annot.Token");
    jcasType.ll_cas.ll_setIntValue(addr, ((Token_Type)jcasType).casFeatCode_depid, v);}    
   
    
  //*--------------*
  //* Feature: dephead

  /** getter for dephead - gets Head token in dependency parse (refers to ID in the same sentence).
   * @generated
   * @return value of the feature 
   */
  public int getDephead() {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_dephead == null)
      jcasType.jcas.throwFeatMissing("dephead", "werti.uima.types.annot.Token");
    return jcasType.ll_cas.ll_getIntValue(addr, ((Token_Type)jcasType).casFeatCode_dephead);}
    
  /** setter for dephead - sets Head token in dependency parse (refers to ID in the same sentence). 
   * @generated
   * @param v value to set into the feature 
   */
  public void setDephead(int v) {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_dephead == null)
      jcasType.jcas.throwFeatMissing("dephead", "werti.uima.types.annot.Token");
    jcasType.ll_cas.ll_setIntValue(addr, ((Token_Type)jcasType).casFeatCode_dephead, v);}    
   
    
  //*--------------*
  //* Feature: deprel

  /** getter for deprel - gets Relation to head token in dependency parse.
   * @generated
   * @return value of the feature 
   */
  public String getDeprel() {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_deprel == null)
      jcasType.jcas.throwFeatMissing("deprel", "werti.uima.types.annot.Token");
    return jcasType.ll_cas.ll_getStringValue(addr, ((Token_Type)jcasType).casFeatCode_deprel);}
    
  /** setter for deprel - sets Relation to head token in dependency parse. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setDeprel(String v) {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_deprel == null)
      jcasType.jcas.throwFeatMissing("deprel", "werti.uima.types.annot.Token");
    jcasType.ll_cas.ll_setStringValue(addr, ((Token_Type)jcasType).casFeatCode_deprel, v);}    
   
    
  //*--------------*
  //* Feature: maltdepid

  /** getter for maltdepid - gets Token ID used to stored dependency parse (should we use position instead?).
   * @generated
   * @return value of the feature 
   */
  public int getMaltdepid() {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_maltdepid == null)
      jcasType.jcas.throwFeatMissing("maltdepid", "werti.uima.types.annot.Token");
    return jcasType.ll_cas.ll_getIntValue(addr, ((Token_Type)jcasType).casFeatCode_maltdepid);}
    
  /** setter for maltdepid - sets Token ID used to stored dependency parse (should we use position instead?). 
   * @generated
   * @param v value to set into the feature 
   */
  public void setMaltdepid(int v) {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_maltdepid == null)
      jcasType.jcas.throwFeatMissing("maltdepid", "werti.uima.types.annot.Token");
    jcasType.ll_cas.ll_setIntValue(addr, ((Token_Type)jcasType).casFeatCode_maltdepid, v);}    
   
    
  //*--------------*
  //* Feature: maltdephead

  /** getter for maltdephead - gets Head token in dependency parse (refers to ID in the same sentence).
   * @generated
   * @return value of the feature 
   */
  public int getMaltdephead() {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_maltdephead == null)
      jcasType.jcas.throwFeatMissing("maltdephead", "werti.uima.types.annot.Token");
    return jcasType.ll_cas.ll_getIntValue(addr, ((Token_Type)jcasType).casFeatCode_maltdephead);}
    
  /** setter for maltdephead - sets Head token in dependency parse (refers to ID in the same sentence). 
   * @generated
   * @param v value to set into the feature 
   */
  public void setMaltdephead(int v) {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_maltdephead == null)
      jcasType.jcas.throwFeatMissing("maltdephead", "werti.uima.types.annot.Token");
    jcasType.ll_cas.ll_setIntValue(addr, ((Token_Type)jcasType).casFeatCode_maltdephead, v);}    
   
    
  //*--------------*
  //* Feature: maltdeprel

  /** getter for maltdeprel - gets Relation to head token in dependency parse.
   * @generated
   * @return value of the feature 
   */
  public String getMaltdeprel() {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_maltdeprel == null)
      jcasType.jcas.throwFeatMissing("maltdeprel", "werti.uima.types.annot.Token");
    return jcasType.ll_cas.ll_getStringValue(addr, ((Token_Type)jcasType).casFeatCode_maltdeprel);}
    
  /** setter for maltdeprel - sets Relation to head token in dependency parse. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setMaltdeprel(String v) {
    if (Token_Type.featOkTst && ((Token_Type)jcasType).casFeat_maltdeprel == null)
      jcasType.jcas.throwFeatMissing("maltdeprel", "werti.uima.types.annot.Token");
    jcasType.ll_cas.ll_setStringValue(addr, ((Token_Type)jcasType).casFeatCode_maltdeprel, v);}    
  }

    