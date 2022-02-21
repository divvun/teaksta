

/* First created by JCasGen Mon Feb 21 10:32:01 CET 2022 */
package werti.uima.types.annot;

import org.apache.uima.jcas.JCas; 
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.jcas.cas.TOP_Type;

import org.apache.uima.jcas.cas.StringList;
import org.apache.uima.jcas.tcas.Annotation;


/** A sentence in natural language derived from plain text and HTML features.
 * Updated by JCasGen Mon Feb 21 10:32:01 CET 2022
 * XML source: /home/trond/gt/main/apps/teaksta/sme/desc/vislcg3TypeSystem.xml
 * @generated */
public class SentenceAnnotation extends Annotation {
  /** @generated
   * @ordered 
   */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = JCasRegistry.register(SentenceAnnotation.class);
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
  protected SentenceAnnotation() {/* intentionally empty block */}
    
  /** Internal - constructor used by generator 
   * @generated
   * @param addr low level Feature Structure reference
   * @param type the type of this Feature Structure 
   */
  public SentenceAnnotation(int addr, TOP_Type type) {
    super(addr, type);
    readObject();
  }
  
  /** @generated
   * @param jcas JCas to which this Feature Structure belongs 
   */
  public SentenceAnnotation(JCas jcas) {
    super(jcas);
    readObject();   
  } 

  /** @generated
   * @param jcas JCas to which this Feature Structure belongs
   * @param begin offset to the begin spot in the SofA
   * @param end offset to the end spot in the SofA 
  */  
  public SentenceAnnotation(JCas jcas, int begin, int end) {
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
  //* Feature: coherence

  /** getter for coherence - gets The coherence of this sentence. How many html tags interefere?
   * @generated
   * @return value of the feature 
   */
  public double getCoherence() {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_coherence == null)
      jcasType.jcas.throwFeatMissing("coherence", "werti.uima.types.annot.SentenceAnnotation");
    return jcasType.ll_cas.ll_getDoubleValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_coherence);}
    
  /** setter for coherence - sets The coherence of this sentence. How many html tags interefere? 
   * @generated
   * @param v value to set into the feature 
   */
  public void setCoherence(double v) {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_coherence == null)
      jcasType.jcas.throwFeatMissing("coherence", "werti.uima.types.annot.SentenceAnnotation");
    jcasType.ll_cas.ll_setDoubleValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_coherence, v);}    
   
    
  //*--------------*
  //* Feature: sexp

  /** getter for sexp - gets The Sexp of the parse of this sentence.
   * @generated
   * @return value of the feature 
   */
  public String getSexp() {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_sexp == null)
      jcasType.jcas.throwFeatMissing("sexp", "werti.uima.types.annot.SentenceAnnotation");
    return jcasType.ll_cas.ll_getStringValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_sexp);}
    
  /** setter for sexp - sets The Sexp of the parse of this sentence. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setSexp(String v) {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_sexp == null)
      jcasType.jcas.throwFeatMissing("sexp", "werti.uima.types.annot.SentenceAnnotation");
    jcasType.ll_cas.ll_setStringValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_sexp, v);}    
   
    
  //*--------------*
  //* Feature: hasdepparse

  /** getter for hasdepparse - gets A flag that marks whether this sentence made it past the filter and has a parse.
   * @generated
   * @return value of the feature 
   */
  public boolean getHasdepparse() {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_hasdepparse == null)
      jcasType.jcas.throwFeatMissing("hasdepparse", "werti.uima.types.annot.SentenceAnnotation");
    return jcasType.ll_cas.ll_getBooleanValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_hasdepparse);}
    
  /** setter for hasdepparse - sets A flag that marks whether this sentence made it past the filter and has a parse. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setHasdepparse(boolean v) {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_hasdepparse == null)
      jcasType.jcas.throwFeatMissing("hasdepparse", "werti.uima.types.annot.SentenceAnnotation");
    jcasType.ll_cas.ll_setBooleanValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_hasdepparse, v);}    
   
    
  //*--------------*
  //* Feature: activeconversion

  /** getter for activeconversion - gets The active version of a passive sentence.
   * @generated
   * @return value of the feature 
   */
  public String getActiveconversion() {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_activeconversion == null)
      jcasType.jcas.throwFeatMissing("activeconversion", "werti.uima.types.annot.SentenceAnnotation");
    return jcasType.ll_cas.ll_getStringValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_activeconversion);}
    
  /** setter for activeconversion - sets The active version of a passive sentence. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setActiveconversion(String v) {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_activeconversion == null)
      jcasType.jcas.throwFeatMissing("activeconversion", "werti.uima.types.annot.SentenceAnnotation");
    jcasType.ll_cas.ll_setStringValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_activeconversion, v);}    
   
    
  //*--------------*
  //* Feature: passiveconversion

  /** getter for passiveconversion - gets The passive version of an active sentence.
   * @generated
   * @return value of the feature 
   */
  public String getPassiveconversion() {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_passiveconversion == null)
      jcasType.jcas.throwFeatMissing("passiveconversion", "werti.uima.types.annot.SentenceAnnotation");
    return jcasType.ll_cas.ll_getStringValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_passiveconversion);}
    
  /** setter for passiveconversion - sets The passive version of an active sentence. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setPassiveconversion(String v) {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_passiveconversion == null)
      jcasType.jcas.throwFeatMissing("passiveconversion", "werti.uima.types.annot.SentenceAnnotation");
    jcasType.ll_cas.ll_setStringValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_passiveconversion, v);}    
   
    
  //*--------------*
  //* Feature: graphs_debug

  /** getter for graphs_debug - gets An array of sentence of file location for sentence parses that belong to this sentence.
   * @generated
   * @return value of the feature 
   */
  public StringList getGraphs_debug() {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_graphs_debug == null)
      jcasType.jcas.throwFeatMissing("graphs_debug", "werti.uima.types.annot.SentenceAnnotation");
    return (StringList)(jcasType.ll_cas.ll_getFSForRef(jcasType.ll_cas.ll_getRefValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_graphs_debug)));}
    
  /** setter for graphs_debug - sets An array of sentence of file location for sentence parses that belong to this sentence. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setGraphs_debug(StringList v) {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_graphs_debug == null)
      jcasType.jcas.throwFeatMissing("graphs_debug", "werti.uima.types.annot.SentenceAnnotation");
    jcasType.ll_cas.ll_setRefValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_graphs_debug, jcasType.ll_cas.ll_getFSRef(v));}    
   
    
  //*--------------*
  //* Feature: passiveConversionStrategy

  /** getter for passiveConversionStrategy - gets 
   * @generated
   * @return value of the feature 
   */
  public String getPassiveConversionStrategy() {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_passiveConversionStrategy == null)
      jcasType.jcas.throwFeatMissing("passiveConversionStrategy", "werti.uima.types.annot.SentenceAnnotation");
    return jcasType.ll_cas.ll_getStringValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_passiveConversionStrategy);}
    
  /** setter for passiveConversionStrategy - sets  
   * @generated
   * @param v value to set into the feature 
   */
  public void setPassiveConversionStrategy(String v) {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_passiveConversionStrategy == null)
      jcasType.jcas.throwFeatMissing("passiveConversionStrategy", "werti.uima.types.annot.SentenceAnnotation");
    jcasType.ll_cas.ll_setStringValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_passiveConversionStrategy, v);}    
   
    
  //*--------------*
  //* Feature: parseCandidate

  /** getter for parseCandidate - gets Indicates that a sentence should be parsed further down the pipeline.
   * @generated
   * @return value of the feature 
   */
  public boolean getParseCandidate() {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_parseCandidate == null)
      jcasType.jcas.throwFeatMissing("parseCandidate", "werti.uima.types.annot.SentenceAnnotation");
    return jcasType.ll_cas.ll_getBooleanValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_parseCandidate);}
    
  /** setter for parseCandidate - sets Indicates that a sentence should be parsed further down the pipeline. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setParseCandidate(boolean v) {
    if (SentenceAnnotation_Type.featOkTst && ((SentenceAnnotation_Type)jcasType).casFeat_parseCandidate == null)
      jcasType.jcas.throwFeatMissing("parseCandidate", "werti.uima.types.annot.SentenceAnnotation");
    jcasType.ll_cas.ll_setBooleanValue(addr, ((SentenceAnnotation_Type)jcasType).casFeatCode_parseCandidate, v);}    
  }

    