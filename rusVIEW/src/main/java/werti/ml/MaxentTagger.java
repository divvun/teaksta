package werti.ml;

///////////////////////////////////////////////////////////////////////////////
//Copyright (C) 2001 Chieu Hai Leong and Jason Baldridge
//
//This library is free software; you can redistribute it and/or
//modify it under the terms of the GNU Lesser General Public
//License as published by the Free Software Foundation; either
//version 2.1 of the License, or (at your option) any later version.
//
//This library is distributed in the hope that it will be useful,
//but WITHOUT ANY WARRANTY; without even the implied warranty of
//MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//GNU Lesser General Public License for more details.
//
//You should have received a copy of the GNU Lesser General Public
//License along with this program; if not, write to the Free Software
//Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA  02111-1307, USA.
//////////////////////////////////////////////////////////////////////////////



/**
 * Simplified version of Predict.java from maxent used to tag provided data.
 * 
 * @author Jason Baldridge
 * @author Adriane Boyd
 */
public class MaxentTagger {
	//private MaxentModel _model;

	//public MaxentTagger(String modelFileName) throws IOException {
	//	_model = new SuffixSensitiveGISModelReader(new File(
	//			modelFileName)).getModel();
	//}

	//private String eval(String predicates) {
	//	String[] contexts = predicates.split(" ");
	//	double[] ocs;
	//	ocs = _model.eval(contexts);
	//	return _model.getBestOutcome(ocs);
	//}

	///**
	// * Tag data provided in list containing one item per element, 
	// * space-separated features.
	// * 
	// * @return list of tags for each line
	// */
	//public List<String> tag(List<String> data) {
	//	List<String> tags = new ArrayList<String>();
	//	
	//	for (String s : data) {
	//		tags.add(eval(s));
	//	}
	//	
	//	return tags;
	//}
}
