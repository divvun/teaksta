
/* First created by JCasGen Tue Nov 18 13:56:47 CET 2014 */
package werti.uima.types.global;

import org.apache.uima.jcas.JCas;
import org.apache.uima.jcas.JCasRegistry;
import org.apache.uima.cas.impl.CASImpl;
import org.apache.uima.cas.impl.FSGenerator;
import org.apache.uima.cas.FeatureStructure;
import org.apache.uima.cas.impl.TypeImpl;
import org.apache.uima.cas.Type;
import org.apache.uima.cas.impl.FeatureImpl;
import org.apache.uima.cas.Feature;
import org.apache.uima.jcas.tcas.DocumentAnnotation_Type;

/** The enhancement ID of this CAS.
 * Updated by JCasGen Tue Nov 18 13:56:47 CET 2014
 * @generated */
public class EnhancementId_Type extends DocumentAnnotation_Type {
  /** @generated */
  protected FSGenerator getFSGenerator() {return fsGenerator;}
  /** @generated */
  private final FSGenerator fsGenerator = 
    new FSGenerator() {
      public FeatureStructure createFS(int addr, CASImpl cas) {
  			 if (EnhancementId_Type.this.useExistingInstance) {
  			   // Return eq fs instance if already created
  		     FeatureStructure fs = EnhancementId_Type.this.jcas.getJfsFromCaddr(addr);
  		     if (null == fs) {
  		       fs = new EnhancementId(addr, EnhancementId_Type.this);
  			   EnhancementId_Type.this.jcas.putJfsFromCaddr(addr, fs);
  			   return fs;
  		     }
  		     return fs;
        } else return new EnhancementId(addr, EnhancementId_Type.this);
  	  }
    };
  /** @generated */
  public final static int typeIndexID = EnhancementId.typeIndexID;
  /** @generated 
     @modifiable */
  public final static boolean featOkTst = JCasRegistry.getFeatOkTst("werti.uima.types.global.EnhancementId");
 
  /** @generated */
  final Feature casFeat_enhId;
  /** @generated */
  final int     casFeatCode_enhId;
  /** @generated */ 
  public long getEnhId(int addr) {
        if (featOkTst && casFeat_enhId == null)
      jcas.throwFeatMissing("enhId", "werti.uima.types.global.EnhancementId");
    return ll_cas.ll_getLongValue(addr, casFeatCode_enhId);
  }
  /** @generated */    
  public void setEnhId(int addr, long v) {
        if (featOkTst && casFeat_enhId == null)
      jcas.throwFeatMissing("enhId", "werti.uima.types.global.EnhancementId");
    ll_cas.ll_setLongValue(addr, casFeatCode_enhId, v);}
    
  



  /** initialize variables to correspond with Cas Type and Features
	* @generated */
  public EnhancementId_Type(JCas jcas, Type casType) {
    super(jcas, casType);
    casImpl.getFSClassRegistry().addGeneratorForType((TypeImpl)this.casType, getFSGenerator());

 
    casFeat_enhId = jcas.getRequiredFeatureDE(casType, "enhId", "uima.cas.Long", featOkTst);
    casFeatCode_enhId  = (null == casFeat_enhId) ? JCas.INVALID_FEATURE_CODE : ((FeatureImpl)casFeat_enhId).getCode();

  }
}



    