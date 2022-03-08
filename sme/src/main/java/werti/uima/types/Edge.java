

/* First created by JCasGen Tue Mar 08 10:25:10 CET 2022 */
package werti.uima.types;

import org.apache.uima.jcas.JCas; 
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.jcas.cas.TOP_Type;

import org.apache.uima.jcas.cas.TOP;


/** Marks a unidirectional edge in a graph. Only the target node is represented, and the name. Edges are supposed to be fields of nodes.
 * Updated by JCasGen Tue Mar 08 10:25:10 CET 2022
 * XML source: /home/boerre/repos/langtech/apps/teaksta/sme/desc/vislcg3TypeSystem.xml
 * @generated */
public class Edge extends TOP {
  /** @generated
   * @ordered 
   */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = JCasRegistry.register(Edge.class);
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
  protected Edge() {/* intentionally empty block */}
    
  /** Internal - constructor used by generator 
   * @generated
   * @param addr low level Feature Structure reference
   * @param type the type of this Feature Structure 
   */
  public Edge(int addr, TOP_Type type) {
    super(addr, type);
    readObject();
  }
  
  /** @generated
   * @param jcas JCas to which this Feature Structure belongs 
   */
  public Edge(JCas jcas) {
    super(jcas);
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
  //* Feature: label

  /** getter for label - gets The edge's name.
   * @generated
   * @return value of the feature 
   */
  public String getLabel() {
    if (Edge_Type.featOkTst && ((Edge_Type)jcasType).casFeat_label == null)
      jcasType.jcas.throwFeatMissing("label", "werti.uima.types.Edge");
    return jcasType.ll_cas.ll_getStringValue(addr, ((Edge_Type)jcasType).casFeatCode_label);}
    
  /** setter for label - sets The edge's name. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setLabel(String v) {
    if (Edge_Type.featOkTst && ((Edge_Type)jcasType).casFeat_label == null)
      jcasType.jcas.throwFeatMissing("label", "werti.uima.types.Edge");
    jcasType.ll_cas.ll_setStringValue(addr, ((Edge_Type)jcasType).casFeatCode_label, v);}    
   
    
  //*--------------*
  //* Feature: target

  /** getter for target - gets The edge's target token annotation.
   * @generated
   * @return value of the feature 
   */
  public Node getTarget() {
    if (Edge_Type.featOkTst && ((Edge_Type)jcasType).casFeat_target == null)
      jcasType.jcas.throwFeatMissing("target", "werti.uima.types.Edge");
    return (Node)(jcasType.ll_cas.ll_getFSForRef(jcasType.ll_cas.ll_getRefValue(addr, ((Edge_Type)jcasType).casFeatCode_target)));}
    
  /** setter for target - sets The edge's target token annotation. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setTarget(Node v) {
    if (Edge_Type.featOkTst && ((Edge_Type)jcasType).casFeat_target == null)
      jcasType.jcas.throwFeatMissing("target", "werti.uima.types.Edge");
    jcasType.ll_cas.ll_setRefValue(addr, ((Edge_Type)jcasType).casFeatCode_target, jcasType.ll_cas.ll_getFSRef(v));}    
  }

    