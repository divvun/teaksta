

/* First created by JCasGen Mon Feb 21 10:32:01 CET 2022 */
package werti.uima.types;

import org.apache.uima.jcas.JCas; 
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.jcas.cas.TOP_Type;

import org.apache.uima.jcas.cas.FSList;
import org.apache.uima.jcas.cas.TOP;
import werti.uima.types.annot.Token;


/** A node annotation, representing both leaf nodes of a graph, as well as internal nodes.
        Note that this node type can represent n-ary circular graphs, including multiple parent nodes. Any restriction to this, if it is desired, should originate from the implementation.
 * Updated by JCasGen Mon Feb 21 10:32:01 CET 2022
 * XML source: /home/trond/gt/main/apps/teaksta/sme/desc/vislcg3TypeSystem.xml
 * @generated */
public class Node extends TOP {
  /** @generated
   * @ordered 
   */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = JCasRegistry.register(Node.class);
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
  protected Node() {/* intentionally empty block */}
    
  /** Internal - constructor used by generator 
   * @generated
   * @param addr low level Feature Structure reference
   * @param type the type of this Feature Structure 
   */
  public Node(int addr, TOP_Type type) {
    super(addr, type);
    readObject();
  }
  
  /** @generated
   * @param jcas JCas to which this Feature Structure belongs 
   */
  public Node(JCas jcas) {
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
  //* Feature: token

  /** getter for token - gets The token this edge represents, or null. In case this node represents a token, this will be a Token, otherwise null.
   * @generated
   * @return value of the feature 
   */
  public Token getToken() {
    if (Node_Type.featOkTst && ((Node_Type)jcasType).casFeat_token == null)
      jcasType.jcas.throwFeatMissing("token", "werti.uima.types.Node");
    return (Token)(jcasType.ll_cas.ll_getFSForRef(jcasType.ll_cas.ll_getRefValue(addr, ((Node_Type)jcasType).casFeatCode_token)));}
    
  /** setter for token - sets The token this edge represents, or null. In case this node represents a token, this will be a Token, otherwise null. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setToken(Token v) {
    if (Node_Type.featOkTst && ((Node_Type)jcasType).casFeat_token == null)
      jcasType.jcas.throwFeatMissing("token", "werti.uima.types.Node");
    jcasType.ll_cas.ll_setRefValue(addr, ((Node_Type)jcasType).casFeatCode_token, jcasType.ll_cas.ll_getFSRef(v));}    
   
    
  //*--------------*
  //* Feature: parents

  /** getter for parents - gets A list of edges representing links to the node's parents.
   * @generated
   * @return value of the feature 
   */
  public FSList getParents() {
    if (Node_Type.featOkTst && ((Node_Type)jcasType).casFeat_parents == null)
      jcasType.jcas.throwFeatMissing("parents", "werti.uima.types.Node");
    return (FSList)(jcasType.ll_cas.ll_getFSForRef(jcasType.ll_cas.ll_getRefValue(addr, ((Node_Type)jcasType).casFeatCode_parents)));}
    
  /** setter for parents - sets A list of edges representing links to the node's parents. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setParents(FSList v) {
    if (Node_Type.featOkTst && ((Node_Type)jcasType).casFeat_parents == null)
      jcasType.jcas.throwFeatMissing("parents", "werti.uima.types.Node");
    jcasType.ll_cas.ll_setRefValue(addr, ((Node_Type)jcasType).casFeatCode_parents, jcasType.ll_cas.ll_getFSRef(v));}    
   
    
  //*--------------*
  //* Feature: children

  /** getter for children - gets A list of edges, representing the node's children.
   * @generated
   * @return value of the feature 
   */
  public FSList getChildren() {
    if (Node_Type.featOkTst && ((Node_Type)jcasType).casFeat_children == null)
      jcasType.jcas.throwFeatMissing("children", "werti.uima.types.Node");
    return (FSList)(jcasType.ll_cas.ll_getFSForRef(jcasType.ll_cas.ll_getRefValue(addr, ((Node_Type)jcasType).casFeatCode_children)));}
    
  /** setter for children - sets A list of edges, representing the node's children. 
   * @generated
   * @param v value to set into the feature 
   */
  public void setChildren(FSList v) {
    if (Node_Type.featOkTst && ((Node_Type)jcasType).casFeat_children == null)
      jcasType.jcas.throwFeatMissing("children", "werti.uima.types.Node");
    jcasType.ll_cas.ll_setRefValue(addr, ((Node_Type)jcasType).casFeatCode_children, jcasType.ll_cas.ll_getFSRef(v));}    
  }

    