wertiview.ns(function() {
	wertiview.ruswordstress = {

	// maximum number of items in combobox in mc
	MAX_MC: 5,
	// actual number of items in combobox in mc (value is overridden below)
	maxLength: 5,
	
	// candidates for mc options presented to user
	types: [],
	hitList: [],
		
	remove: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ 
			return new jQuery.fn.init(selector,contextDoc||window.content.document); 
			};
		$.fn = $.prototype = jQuery.fn;

		$('body').undelegate('span.wertiviewtoken', 'click', wertiview.ruswordstress.clickHandler);
		$('body').undelegate('select.wertiviewinput', 'change', wertiview.ruswordstress.mcInputHandler);
		$('body').undelegate('span.wertiviewhint', 'click', wertiview.ruswordstress.mcHintHandler);
		$('body').undelegate('input.wertiviewinput', 'change', wertiview.ruswordstress.mcInputHandler);
		$('body').undelegate('input.wertiviewhint', 'click', wertiview.ruswordstress.mcHintHandler);  // was: span.wertiviewhint
		
		// replace the words with stress inside the boxes reserved for user input with the original text
		$('.wertiviewinput').each( function() {
			$(this).replaceWith($(this).data('wertivieworiginaltext'));
		});
		
		// replace the words with stress correctly answered by the user with the original text
		$('.clozeStyleCorrect').each( function() {
			$(this).replaceWith($(this).data('wertivieworiginaltext'));
		});
		
		// replace the words with stress with the original text
		$('.wordWithStress').each( function() {
			$(this).replaceWith($(this).data('wertivieworiginaltext'));
		});
		
		// replace the words with stress selected in the colorize activity with the original text
		$('.colorizeWordWithStress').each( function() {
			$(this).replaceWith($(this).attr('wertivieworiginaltext'));
		});
		
		$('span.wertiviewbaseform').remove();
		$('.wertiviewhint').remove();
	},

	colorize: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ 
			return new jQuery.fn.init(selector,contextDoc||window.content.document); 
			};
		$.fn = $.prototype = jQuery.fn;

		// get potential spans
		var $hits = $('span.wertiviewhit');
		
		// replace the original words with the words with stress
		$hits.each( function() {
			if(!!$(this).attr('wordwithstress')){
				// retrieve the original word
				var originalWord = $(this).text();
				// determine capitalization type: type 0, no caps; type 1, all caps; type 2, first letter cap
				var capType = wertiview.lib.detectCapitalization(originalWord);
				// make sure the word with stress is capitalized correctly (like the original text)
				var wordWithStress = wertiview.lib.matchCapitalization($(this).attr('wordwithstress'), capType);
				
//				// add color to the stressed vowel
//				
//				var stressPattern =/ё|Ё|(а|е|и|о|у|ы|э|ю|я|А|Е|И|О|У|Ы|Э|Ю|Я)(\u0301|\u0300)/g;
//				
//				var vowelWithStressArr = wordWithStress.match(stressPattern);
//				
//				jQuery.each(vowelWithStressArr, function( index, value ) {
//					
//					var $colorizedVowelWithStress = $("<span>");
//					
//					$colorizedVowelWithStress.css('color', 'red');
//					
//					$colorizedVowelWithStress.text(value);
//					
//					wordWithStress = wordWithStress.replace(value, $colorizedVowelWithStress[0].outerHTML);		
//				});
				
				// change the original word to the word with stress
				$(this).html(wordWithStress);
			}
		});

	},
	
	colorizeSpan: function(span, topic, index) {
		var hits = span.find('span.wertiviewhit');
		
		hits.addClass('colorizeWordWithStress');
		
		hits.each( function( index, value ) {
			// retrieve the original word
			var originalWord = value.innerHTML;
			// save the original text in an attribute
			value.setAttribute('wertivieworiginaltext', originalWord);
			// determine capitalization type: type 0, no caps; type 1, all caps; type 2, first letter cap
			var capType = wertiview.lib.detectCapitalization(originalWord);
			// make sure the word with stress is capitalized correctly (like the original text)
			var wordWithStress = wertiview.lib.matchCapitalization(value.getAttribute("wordwithstress"), capType);
			// change the original word to the word with stress
			value.innerHTML = wordWithStress;
		});
	},

	click: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ 
			return new jQuery.fn.init(selector,contextDoc||window.content.document); 
			};
		$.fn = $.prototype = jQuery.fn;
		
		// get potential spans
		var $hits = $('span.wertiviewhit');
		
		// replace the original words with the words with stress
		$hits.each( function() {
			var originalWord = $(this).text();
			// determine capitalization type: type 0, no caps; type 1, all caps; type 2, first letter cap
			var capType = wertiview.lib.detectCapitalization(originalWord);
			// make sure the word with stress is capitalized correctly (like the original text)
			var wordWithStress = wertiview.lib.matchCapitalization($(this).attr('wordwithstress'), capType);
			
			//if(originalWord === "ее"){
				// create new span tag for the word with stress
				$text = $("<span>");
				$text.addClass('wertiview');
				$text.addClass('wordWithStress');
				
				// add span markup to each vowel using the class clickVowel
				// if the vowel has stress add class withStress and store the stress marker
				
				var vowelPattern =/ё|Ё|а|е|и|о|у|ы|э|ю|я|А|Е|И|О|У|Ы|Э|Ю|Я/g;
				
				// array containing all possible vowel matches inside the word with stress
				var vowelArr = wordWithStress.match(vowelPattern);
				
				var previousEnd = 0;			
					
				var startIndexOriginal = 0;
				
				var startIndexWordWithStress = 0;
				
				var correctWordWithSpans = "";
				
				jQuery.each(vowelArr, function( index, value ) {
					
					//alert("Entered loop");
					
					//alert("The current vowel is =" + value);
										
					var indexOfOriginalMatch = originalWord.indexOf(value, startIndexOriginal);
					
					// take care of the special case when in the original the letter "е" was used instead of "ё"
					if(indexOfOriginalMatch === -1){
						var correctedVowel = value.replace("ё","е").replace("Ё","Е");
						indexOfOriginalMatch = originalWord.indexOf(correctedVowel, startIndexOriginal);
					}
					
					//alert("The vowel of the original word is at index ="+ indexOfOriginalMatch);
					
					var currentIndexOfMatch = wordWithStress.indexOf(value, startIndexWordWithStress);
					
					//alert("The vowel is at index =" + currentIndexOfMatch);
					
					var firstStressMarker = '\u0301';
					
					var indexOfFirstStressedVowel = wordWithStress.indexOf(value + firstStressMarker, startIndexWordWithStress);
					
					//alert("The index of a potential primary stress vowel =" + indexOfFirstStressedVowel);
					
					var secondStressMarker = '\u0300';
					
					var indexOfSecondStressedVowel = wordWithStress.indexOf(value + secondStressMarker, startIndexWordWithStress);
					
					//alert("The index of a potential secondary stress vowel =" + indexOfSecondStressedVowel);
					
					var $vowelSpan = $("<span>");
					
					$vowelSpan.addClass('clickVowel');
					
					// add the class withStress to vowels that have stress and save the stress marker
					if(currentIndexOfMatch === indexOfFirstStressedVowel){
						//alert("This vowel is with stress =" + value);
						$vowelSpan.addClass('withStress');
						$vowelSpan.attr('stressedVowel', value + firstStressMarker);
					}
					else if(currentIndexOfMatch === indexOfSecondStressedVowel){
						//alert("This vowel is with stress =" + value);
						$vowelSpan.addClass('withStress');
						$vowelSpan.attr('stressedVowel', value + secondStressMarker);
					}
					else if(value === "ё" ||
							value === "Ё"){
						//alert("This vowel is with stress =" + value);
						$vowelSpan.addClass('withStress');
					}					
					$vowelSpan.text(value);
					
					var matchStart = indexOfOriginalMatch;
					
					var matchEnd = matchStart + 1;
					
					//alert(value + " begins at pos =" + matchStart + " and ends at=" + matchEnd);
					
					//alert("The part before the next vowel span is=" + originalWord.slice(previousEnd, matchStart));
					
					var leftSide = correctWordWithSpans + originalWord.slice(previousEnd, matchStart);
					
					var vowelSpanHtml = $vowelSpan[0].outerHTML;
					
					// dynamically construct the text inside a word with stress
					// vowels are surrounded by span tags
					if(index === vowelArr.length - 1){ // the last vowel match
						//alert("In the last iteration the leftside is =" + leftSide);
						//alert("The html markup is=" + vowelSpanHtml);
						//alert("The rest of the word is=" + originalWord.slice(matchEnd));
						correctWordWithSpans = leftSide + vowelSpanHtml + originalWord.slice(matchEnd);
					}
					else{ // any other vowel match
						correctWordWithSpans = leftSide + vowelSpanHtml;
					}
					
					//alert("Currently the correct word with span is =" + correctWordWithSpans);
					
					// for the next match...
					
					// save the vowel match end from the current match
					previousEnd = matchEnd;
					
					// save the index that indicates where to start to search in the original word
					startIndexOriginal = matchEnd;
					
					// save the index that indicates where to start to search in the word with stress
					startIndexWordWithStress = currentIndexOfMatch + 1;
					
					// to search the remaining string for a vowel match
				});
			
				//alert("In the end the correct word with span is =" + correctWordWithSpans);
				
				// put the word with the spans for the vowels, into the span tag
				$text.html(correctWordWithSpans);
				// save the original text in a hidden field
				$text.data('wertivieworiginaltext', originalWord);
				// replace the wertiviewhit with this span
				$(this).replaceWith($text);
			//}			
		});

		// change all clickVowel spans to mouseover pointer
		$('span.clickVowel').css({'cursor': 'pointer'}); 
		
		// handle click
		$('body').delegate('span.clickVowel', 'click', {context: contextDoc}, wertiview.ruswordstress.clickHandler); 
	},

	clickHandler: function(event) {
		var contextDoc = event.data.context;

		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ 
			return new jQuery.fn.init(selector,contextDoc||window.content.document); 
			};
		$.fn = $.prototype = jQuery.fn;
		
		if($(this).hasClass('withStress')) {  // was: wertiviewhit
			$(this).addClass('clickStyleCorrect');
			$(this).text($(this).attr('stressedVowel'));
		} else {
			$(this).addClass('clickStyleIncorrect');
		} 
		
		return false;
	},
	
	mc: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ 
			return new jQuery.fn.init(selector,contextDoc||window.content.document); 
			};
		$.fn = $.prototype = jQuery.fn;
		
		// get potential spans
		var $hits = $('span.wertiviewhit');
		
		var hitList = [];
		var tokens = [];
		wertiview.ruswordstress.types = [];
		//alert($hits.length+" hits");
		$hits.each( function() {
			hitList.push($(this));
			//alert($(this).text());
			tokens[$(this).text().toLowerCase()] = 1;
		});

		wertiview.ruswordstress.maxLength = wertiview.ruswordstress.MAX_MC;
		wertiview.activity.mc(contextDoc, hitList, 
				wertiview.ruswordstress.mcInputHandler, 
				wertiview.ruswordstress.mcHintHandler, 
				wertiview.ruswordstress.mcGetOptions, 
				wertiview.ruswordstress.mcGetCorrectAnswer);

	},
	
	mcGetOptions: function($hit, capType){
		var options = [];
		var j = 0;
		// Get the list of distractors for the given hit (they are saved as a space-separated list in the attribute "distractors" of the wertiview span tag):
		wertiview.ruswordstress.types = $hit.attr('distractors').split(" ");
	    wertiview.lib.shuffleList(wertiview.ruswordstress.types);
        
        // Add the distractor forms to the options list:
        while (j < wertiview.ruswordstress.types.length && options.length < wertiview.ruswordstress.MAX_MC - 1) {
            // The forms that are homonymous to the correct form are excluded from the list of options:
            if (wertiview.ruswordstress.types[j].toLowerCase() != $hit.attr('wordwithstress').toLowerCase() && wertiview.ruswordstress.types[j] != "") 
            {
               options.push(wertiview.lib.matchCapitalization(wertiview.ruswordstress.types[j], capType)); 
            }
			j++;
		}
		
		options.push(wertiview.lib.matchCapitalization($hit.attr('wordwithstress'), capType));
		wertiview.lib.shuffleList(options);
		return options;
	},
	
	mcGetCorrectAnswer: function($hit, capType){
		return wertiview.lib.matchCapitalization($hit.attr('wordwithstress'), capType);
	},
	
	cloze: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ 
			return new jQuery.fn.init(selector,contextDoc||window.content.document); 
			};
		$.fn = $.prototype = jQuery.fn;
		
		// get potential spans
		var $hits = $('span.wertiviewhit');
		
		$hits.hover(
		// when entering the word...
		function() {
				// retrieve the original word
				var originalWord = $(this).text();
				// determine capitalization type: type 0, no caps; type 1, all caps; type 2, first letter cap
				var capType = wertiview.lib.detectCapitalization(originalWord);
				// make sure the word with stress is capitalized correctly (like the original text)
				var wordWithStress = wertiview.lib.matchCapitalization($(this).attr('wordwithstress'), capType);
				
				// add color to the stressed vowel
				// а́ е́ ё и́ о́ у́ ы́ э́ ю́ я́
		    	// А́ Е́ Ё И́ О́ У́ Ы́ Э́ Ю́ Я́ 
				var stressPattern =/ё|Ё|(а|е|и|о|у|ы|э|ю|я|А|Е|И|О|У|Ы|Э|Ю|Я)(\u0301|\u0300)/g;
				
				var vowelWithStressArr = wordWithStress.match(stressPattern);
				
				jQuery.each(vowelWithStressArr, function( index, value ) {
					
					var $colorizedVowelWithStress = $("<span>");
					
					$colorizedVowelWithStress.css('color', 'red');
					
					$colorizedVowelWithStress.text(value);
					
					wordWithStress = wordWithStress.replace(value, $colorizedVowelWithStress[0].outerHTML);		
				});
				
				// change the original word to the word with stress
				$(this).html(wordWithStress);
				// save the original text in a hidden field
				$(this).data('wertivieworiginaltext', originalWord);
			
		}, 
		// when leaving the word...
		function() {
			// remove the style attribute
			$(this).removeAttr("style");
			// return the text to the original
			$(this).html($(this).data('wertivieworiginaltext'));
		});
	},

	mcInputHandler: function(event) {
		var jQuery = wertiview.jQuery;
		var contextDoc = event.data.context;
		  var $ = function(selector,context){ 
			  return new jQuery.fn.init(selector,contextDoc||window.content.document); 
			  };
		  $.fn = $.prototype = jQuery.fn;

		var nextInput;

		// if the answer is correct, turn into text, else color text within input
		if($(this).val().toLowerCase() == $(this).data('wertiviewanswer').toLowerCase()) {
			$text = $("<span>");
			$text.addClass('wertiview');
			$text.addClass('clozeStyleCorrect');
			$text.text($(this).data('wertiviewanswer'));
			// save the original text in a hidden field
			$text.data('wertivieworiginaltext', $(this).data('wertivieworiginaltext'));
			if($(this).data('wertiviewnexthit')) {   
				nextInput = $(this).data('wertiviewnexthit');
			}
			wertiview.lib.replaceInput($(this).parent(), $text);

		} else {
			$(this).addClass('clozeStyleIncorrect');
		}
		return false;
	},

	mcHintHandler: function(event) {
		var jQuery = wertiview.jQuery;
		var contextDoc = event.data.context;
		  var $ = function(selector,context){ 
			  return new jQuery.fn.init(selector,contextDoc||window.content.document); 
			  };
		  $.fn = $.prototype = jQuery.fn;

		var nextInput;

		// fill in the answer by replacing input with text
		$text = $("<span>");
		$text.addClass('wertiview');
		$text.addClass('clozeStyleProvided');
		$text.text($(this).prev().data('wertiviewanswer'));
		if($(this).prev().data('wertiviewnexthit')) {  
			nextInput = $(this).prev().data('wertiviewnexthit');
		}
		wertiview.lib.replaceInput($(this).parent(), $text);
		
		return false;
	}
	};
}); // REMOVE-WITH-MAVEN-REPLACER-PLUGIN