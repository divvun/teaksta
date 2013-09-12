	wertiview.substantiveplural = {

    // maximum number of instances to turn into exercises (moved to preferences)
	//MAX_CLOZE: 25,
	// maximum number of items in combobox in mc
	MAX_MC: 5,
	// actual number of items in combobox in mc (value is overridden below)
	maxLength: 5,
	
	// candidates for mc options presented to user
	types: [],
	hitList: [],
		
	remove: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
		$.fn = $.prototype = jQuery.fn;

		$('body').undelegate('span.wertiviewtoken', 'click', wertiview.substantiveplural.clickHandler);
		$('body').undelegate('select.wertiviewinput', 'change', wertiview.substantiveplural.clozeInputHandler);
		$('body').undelegate('span.wertiviewhint', 'click', wertiview.substantiveplural.clozeHintHandler);
		$('body').undelegate('input.wertiviewinput', 'change', wertiview.substantiveplural.clozeInputHandler);
		$('body').undelegate('input.wertiviewhint', 'click', wertiview.substantiveplural.clozeHintHandler);  // was: span.wertiviewhint
		
		$('.wertiviewinput').each( function() {
			$(this).replaceWith($(this).data('wertiviewanswer'));
		});
		//$('span.wertiviewbaseform').remove();
		$('.wertiviewhint').remove();
	},

	colorize: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
		$.fn = $.prototype = jQuery.fn;

		$('span.wertiviewSubstantivePlural').addClass('colorizeStyleSubstantivePlural');
	},
	
	colorizeSpan: function(span, topic) {
span.find('span.wertiviewSubstantivePlural').addClass('colorizeStyleSubstantivePlural');
	},

	click: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
		$.fn = $.prototype = jQuery.fn;

		// change all wertiviewtoken spans to mouseover pointer
		$('span.wertiviewtoken').css({'cursor': 'pointer'}); 

		// conjunction markup
		$('span.wertiviewRELEVANT').find('span.wertiviewSubstantivePlural').addClass('colorizeStyleSubstantivePlural');

		// correct cursor inside wertiviewtokens within multi-word spans
		//$('span.wertiviewRELEVANT').find('span.wertiviewconjunction').css({'cursor': 'text'});
		
		// handle click
		$('body').delegate('span.wertiviewtoken', 'click', {context: contextDoc}, wertiview.substantiveplural.clickHandler); 
	},

	clickHandler: function(event) {
		var contextDoc = event.data.context;

		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
		$.fn = $.prototype = jQuery.fn;
		
		if($(this).hasClass('wertiviewSubstantivePlural')) {  // was: wertiviewhit
			$(this).addClass('clickStyleCorrect');
		} else {
			$(this).addClass('clickStyleIncorrect');
		} 
		//$(this).css({'cursor': 'auto'});
        
		// not within a relevant phrase
		/*if ($(this).parents('.wertiviewRELEVANT').length == 0) {
			$(this).addClass('clickStyleIncorrect');
			return false;
		}

		// an already colored conjunction
		var isColored = false;

		if ($(this).hasClass('wertiviewconjunction') || $(this).find('.wertiviewconjunction').length > 0) {
			isColored = true;
		}

		if (isColored) {
			return false;
		}

		// TODO: if this is a clue
		var isClue = false;

		if ($(this).hasClass('wertiviewCLU-BOTHMEANDIFF') || $(this).hasClass('wertiviewCLU-BOTHMEANSAME') || 
				$(this).hasClass('wertiviewCLU-FIXEDEXP') || $(this).hasClass('wertiviewCLU-GERONLY') || 
				$(this).hasClass('wertiviewCLU-INFONLY') || 
				$(this).find('.wertiviewCLU-BOTHMEANDIFF, .wertiviewCLU-BOTHMEANSAME, .wertiviewCLU-FIXEDEXP, .wertiviewCLU-GERONLY, .wertiviewCLU-INFONLY').length > 0) {
			isClue = true;
		}
		
		if (isClue) {
			$(this).addClass('clickStyleCorrect');
		} else {
			$(this).addClass('clickStyleIncorrect');
		}*/
		return false;
	},
	
	mc: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
		$.fn = $.prototype = jQuery.fn;
		
		// get potential spans
		var $hits = $('span.wertiviewSubstantivePlural');
		
		//var hitList = [];
		var tokens = [];
		wertiview.substantiveplural.types = [];
		wertiview.substantiveplural.hitList = [];
		//alert($hits.length+" hits");
		$hits.each( function() {
			wertiview.substantiveplural.hitList.push($(this));
			//alert($(this).text());
			tokens[$(this).text().toLowerCase()] = 1;
		});
		//alert("number of tokens: "+tokens.length);
		//alert("size of hitList: "+wertiview.substantiveplural.hitList.length);
		/*for (word in tokens) {
			wertiview.substantiveplural.types.push(word);
			//alert("word: "+word);
		}*/
		//alert(wertiview.substantiveplural.types.length+" different substantiveplural on page");

		wertiview.substantiveplural.maxLength = wertiview.substantiveplural.MAX_MC;
		/*if (wertiview.substantiveplural.maxLength > wertiview.substantiveplural.types.length) {
			wertiview.substantiveplural.maxLength = wertiview.substantiveplural.types.length;
		}*/

		/* 
		$hits.each( function() {
			// if this is a split infinitive, skip
			if ($(this).find('.wertiviewINFSPLIT').length == 0) {
				var options = $(this).attr('title').split(";");
				// if the infinitive or gerund isn't given in the markup, skip
				for (var j = 0; j < options.length; j++) {
					if (options[j] == 'null') {
						return;
					}
				}
				hitList.push($(this));				
			} 
		}); */

		wertiview.activity.mc(contextDoc, wertiview.substantiveplural.hitList, 
				wertiview.substantiveplural.clozeInputHandler, 
				wertiview.substantiveplural.clozeHintHandler, 
				wertiview.substantiveplural.mcGetOptions, 
				wertiview.substantiveplural.mcGetCorrectAnswer);

	},
	
	mcGetOptions: function($hit, capType){
		var options = [];
		var j = 0;
		// Get the list of distractors for the given hit (they are saved as a space-separated list in the attribute "distractors" of the wertiview span tag):
		wertiview.substantiveplural.types = $hit.attr('distractors').split(" ");
	    wertiview.lib.shuffleList(wertiview.substantiveplural.types);
        
        // Add the distractor forms to the options list:
        while (j < wertiview.substantiveplural.types.length && options.length < wertiview.substantiveplural.MAX_MC - 1) {
            // The forms that are homonymous to the correct form are excluded from the list of options:
            if (wertiview.substantiveplural.types[j] != $hit.text().toLowerCase() && wertiview.substantiveplural.types[j] != "") 
            {
                var homonym = false;
                var k = 0;
                while (k < j) //check for homonymes among the distractors
                { 
                    if (wertiview.substantiveplural.types[k] == wertiview.substantiveplural.types[j])
                        homonym = true;
                    k++;
                } 
                if (!homonym)                 options.push(wertiview.lib.matchCapitalization(wertiview.substantiveplural.types[j], capType)); 
            }
			j++;
		}
		
		options.push(wertiview.lib.matchCapitalization($hit.text(), capType));
		wertiview.lib.shuffleList(options);
		return options;

		//var options = $hit.attr('title').split(";");
		//return options;
	},
	
	mcGetCorrectAnswer: function($hit, capType){
		return $hit.text();
	},
	
	cloze: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
		$.fn = $.prototype = jQuery.fn;
		
		// get potential spans
		//var $hits = $('span.wertiviewRELEVANT').find('span.wertiviewconjunction');
		var $hits = $('span.wertiviewSubstantivePlural');

		var hitList = [];
		$hits.each( function() {
			hitList.push($(this));				
		}); 
		/*$hits.each( function() {
			// if this is a split infinitive, skip
			if ($(this).find('.wertiviewINFSPLIT').length == 0) {
				hitList.push($(this));				
			}
		});*/

		wertiview.activity.cloze(contextDoc, hitList, 
				wertiview.substantiveplural.clozeInputHandler, 
				wertiview.substantiveplural.clozeHintHandler, 
				wertiview.substantiveplural.mcGetCorrectAnswer,
				wertiview.substantiveplural.clozeAddBaseform);
	},
	
	clozeAddBaseform: function($hit, capType, $){
		// create baseform info
		var $baseform = $('<span>');
		$baseform.addClass('clozeStyleBaseform');
		$baseform.addClass('wertiviewbaseform');
		var lemmaform = $hit.attr('lemma');
		if (lemmaform)
		  $baseform.text(' (' + lemmaform + ')');
		  $hit.append($baseform);
	},

	clozeInputHandler: function(event) {
		var jQuery = wertiview.jQuery;
		var contextDoc = event.data.context;
		  var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
		  $.fn = $.prototype = jQuery.fn;

		var nextInput;

		// if the answer is correct, turn into text, else color text within input
		if($(this).val().toLowerCase() == $(this).data('wertiviewanswer').toLowerCase()) {
			$text = $("<span>");
			$text.addClass('wertiview');
			$text.addClass('clozeStyleCorrect');
			$text.text($(this).data('wertiviewanswer'));
			if($(this).data('wertiviewnexthit')) {   
				nextInput = $(this).data('wertiviewnexthit');
			}
			wertiview.lib.replaceInput($(this).parent(), $text);

			/*// focus next input
			if(nextInput) {
				$("#" + nextInput).get(0).focus();
			}*/
		} else {
			$(this).addClass('clozeStyleIncorrect');
		}
	},

	clozeHintHandler: function(event) {
		var jQuery = wertiview.jQuery;
		var contextDoc = event.data.context;
		  var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
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

		/*// focus next input
		if(nextInput) {
			$("#" + nextInput).get(0).focus();
		}*/
		
		return false;
	}
	};

