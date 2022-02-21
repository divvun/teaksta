
/* First created by JCasGen Mon Feb 21 10:32:01 CET 2022 */
package werti.uima.types;

import org.apache.uima.jcas.JCas;
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.cas.impl.TypeImpl;
import org.apache.uima.cas.Type;
import org.apache.uima.cas.impl.FeatureImpl;
import org.apache.uima.cas.Feature;
import org.apache.uima.jcas.cas.TOP_Type;

/** A node annotation, representing both leaf nodes of a graph, as well as internal nodes.
        Note that this node type can represent n-ary circular graphs, including multiple parent nodes. Any restriction to this, if it is desired, should originate from the implementation.
 * Updated by JCasGen Mon Feb 21 10:32:01 CET 2022
 * @generated */
public class Node_Type extends TOP_Type {
  /** @generated */
  @SuppressWarnings ("hiding")
  public final static int typeIndexID = Node.typeIndexID;
  /** @generated 
     @modifiable */
  @SuppressWarnings ("hiding")
  public final static boolean featOkTst = JCasRegistry.getFeatOkTst("werti.uima.types.Node");
 
  /** @generated */
  final Feature casFeat_token;
  /** @generated */
  final int     casFeatCode_token;
  /** @generated
   * @param addr low level Feature Structure reference
   * @return the feature value 
   */ 
  public int getToken(int addr) {
        if (featOkTst && casFeat_token == null)
      jcas.throwFeatMissing("token", "werti.uima.types.Node");
    return ll_cas.ll_getRefValue(addr, casFeatCode_token);
  }
  /** @generated
   * @param addr low level Feature Structure reference
   * @param v value to set 
   */    
  public void setToken(int addr, int v) {
        if (featOkTst && casFeat_token == null)
      jcas.throwFeatMissing("token", "werti.uima.types.Node");
    ll_cas.ll_setRefValue(addr, casFeatCode_token, v);}
    
  
 
  /** @generated */
  final Feature casFeat_parents;
  /** @generated */
  final int     casFeatCode_parents;
  /** @generated
   * @param addr low level Feature Structure reference
   * @return the feature value 
   */ 
  public int getParents(int addr) {
        if (featOkTst && casFeat_parents == null)
      jcas.throwFeatMissing("parents", "werti.uima.types.Node");
    return ll_cas.ll_getRefValue(addr, casFeatCode_parents);
  }
  /** @generated
   * @param addr low level Feature Structure reference
   * @param v value to set 
   */    
  public void setParents(int addr, int v) {
        if (featOkTst && casFeat_parents == null)
      jcas.throwFeatMissing("parents", "werti.uima.types.Node");
    ll_cas.ll_setRefValue(addr, casFeatCode_parents, v);}
    
  
 
  /** @generated */
  final Feature casFeat_children;
  /** @generated */
  final int     casFeatCode_children;
  /** @generated
   * @param addr low level Feature Structure reference
   * @return the feature value 
   */ 
  public int getChildren(int addr) {
        if (featOkTst && casFeat_children == null)
      jcas.throwFeatMissing("children", "werti.uima.types.Node");
    return ll_cas.ll_getRefValue(addr, casFeatCode_children);
  }
  /** @generated
   * @param addr low level Feature Structure reference
   * @param v value to set 
   */    
  public void setChildren(int addr, int v) {
        if (featOkTst && casFeat_children == null)
      jcas.throwFeatMissing("children", "werti.uima.types.Node");
    ll_cas.ll_setRefValue(addr, casFeatCode_children, v);}
    
  



  /** initialize variables to correspond with Cas Type and Features
	 * @generated
	 * @param jcas JCas
	 * @param casType Type 
	 */
  public Node_Type(JCas jcas, Type casType) {
    super(jcas, casType);
    casImpl.getFSClassRegistry().addGeneratorForType((TypeImpl)this.casType, getFSGenerator());

 
    casFeat_token = jcas.getRequiredFeatureDE(casType, "token", "werti.uima.types.annot.Token", featOkTst);
    casFeatCode_token  = (null == casFeat_token) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_token).getCode();

 
    casFeat_parents = jcas.getRequiredFeatureDE(casType, "parents", "uima.cas.FSList", featOkTst);
    casFeatCode_parents  = (null == casFeat_parents) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_parents).getCode();

 
    casFeat_children = jcas.getRequiredFeatureDE(casType, "children", "uima.cas.FSList", featOkTst);
    casFeatCode_children  = (null == casFeat_children) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_children).getCode();

  }
}



    