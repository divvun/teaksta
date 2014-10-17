	wertiview.substantivesingular = {

    // maximum number of instances to turn into exercises (moved to preferences)
	//MAX_CLOZE: 25,
	// maximum number of items in combobox in mc
	MAX_MC: 5,
	// actual number of items in combobox in mc (value is overridden below)
	maxLength: 5,
	
	// candidates for mc options presented to user
	types: [],
	hitList: [],
	
	// the total number of clicks
	totalclicked : 0,
	
	// the total number of correct clicks
	totalresult : 0,
		
	remove: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
		$.fn = $.prototype = jQuery.fn;

		$('body').undelegate('span.wertiviewtoken', 'click', wertiview.substantivesingular.clickHandler);
		$('body').undelegate('select.wertiviewinput', 'change', wertiview.substantivesingular.clozeInputHandler);
		$('body').undelegate('span.wertiviewhint', 'click', wertiview.substantivesingular.clozeHintHandler);
		$('body').undelegate('input.wertiviewinput', 'change', wertiview.substantivesingular.clozeInputHandler);
		$('body').undelegate('input.wertiviewhint', 'click', wertiview.substantivesingular.clozeHintHandler);  // was: span.wertiviewhint
		
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

		$('span.wertiviewSubstantiveSingular').addClass('colorizeStyleSubstantiveSingular');
	},
	
	colorizeSpan: function(span, topic) {
span.find('span.wertiviewSubstantiveSingular').addClass('colorizeStyleSubstantiveSingular');
	},

	click: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
		$.fn = $.prototype = jQuery.fn;

		// change all wertiviewtoken spans to mouseover pointer
		$('span.wertiviewtoken').css({'cursor': 'pointer'});
		var numExercises = $('span.wertiviewSubstantiveSingular').length;
		jQuery.ajax({
            type: "POST",
            url: "http://localhost:8080/teaksta/WERTiServlet", // save data (nr of generated exercises) in a log file, without reloading the page
            data: {nr_of_exercises : numExercises},
            success: function() {
                //display message back to user here
            }
        });

		// conjunction markup
		$('span.wertiviewRELEVANT').find('span.wertiviewSubstantiveSingular').addClass('colorizeStyleSubstantiveSingular');

		// correct cursor inside wertiviewtokens within multi-word spans
		//$('span.wertiviewRELEVANT').find('span.wertiviewconjunction').css({'cursor': 'text'});
		
		// handle click
		$('body').delegate('span.wertiviewtoken', 'click', {context: contextDoc}, wertiview.substantivesingular.clickHandler); 
	},

	clickHandler: function(event) {
		var contextDoc = event.data.context;

		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
		$.fn = $.prototype = jQuery.fn;
		
		if($(this).hasClass('wertiviewSubstantiveSingular')) {  // was: wertiviewhit
			$(this).addClass('clickStyleCorrect');
            wertiview.substantivesingular.updateClickLog(event, "click", 1, $(this).text(), "");
		} else {
			$(this).addClass('clickStyleIncorrect');
			wertiview.substantivesingular.updateClickLog(event, "click", 0, $(this).text(), "");
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
		var $hits = $('span.wertiviewSubstantiveSingular');
		
		//var hitList = [];
		var tokens = [];
		wertiview.substantivesingular.types = [];
		wertiview.substantivesingular.hitList = [];
		//alert($hits.length+" hits");
		$hits.each( function() {
			wertiview.substantivesingular.hitList.push($(this));
			//alert($(this).text());
			tokens[$(this).text().toLowerCase()] = 1;
		});
		//alert("number of tokens: "+tokens.length);
		//alert("size of hitList: "+wertiview.substantivesingular.hitList.length);
		/*for (word in tokens) {
			wertiview.substantivesingular.types.push(word);
			//alert("word: "+word);
		}*/
		//alert(wertiview.substantivesingular.types.length+" different substantivesingular on page");

		wertiview.substantivesingular.maxLength = wertiview.substantivesingular.MAX_MC;
		/*if (wertiview.substantivesingular.maxLength > wertiview.substantivesingular.types.length) {
			wertiview.substantivesingular.maxLength = wertiview.substantivesingular.types.length;
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

		wertiview.activity.mc(contextDoc, wertiview.substantivesingular.hitList, 
				wertiview.substantivesingular.clozeInputHandler, 
				wertiview.substantivesingular.clozeHintHandler, 
				wertiview.substantivesingular.mcGetOptions, 
				wertiview.substantivesingular.mcGetCorrectAnswer);

	},
	
	mcGetOptions: function($hit, capType){
		var options = [];
		var j = 0;
		// Get the list of distractors for the given hit (they are saved as a space-separated list in the attribute "distractors" of the wertiview span tag):
		wertiview.substantivesingular.types = $hit.attr('distractors').split(" ");
	    wertiview.lib.shuffleList(wertiview.substantivesingular.types);
        
        // Add the distractor forms to the options list:
        while (j < wertiview.substantivesingular.types.length && options.length < wertiview.substantivesingular.MAX_MC - 1) {
            // The forms that are homonymous to the correct form are excluded from the list of options:
            if (wertiview.substantivesingular.types[j] != $hit.text().toLowerCase() && wertiview.substantivesingular.types[j] != "") 
            {
                var homonym = false;
                var k = 0;
                while (k < j) //check for homonymes among the distractors
                { 
                    if (wertiview.substantivesingular.types[k] == wertiview.substantivesingular.types[j])
                        homonym = true;
                    k++;
                } 
                if (!homonym)                 options.push(wertiview.lib.matchCapitalization(wertiview.substantivesingular.types[j], capType)); 
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
		var $hits = $('span.wertiviewSubstantiveSingular');

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
				wertiview.substantivesingular.clozeInputHandler, 
				wertiview.substantivesingular.clozeHintHandler, 
				wertiview.substantivesingular.mcGetCorrectAnswer,
				wertiview.substantivesingular.clozeAddBaseform);
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
			wertiview.substantivesingular.updateClickLog(event, "cloze", 1, $(this).val(), $(this).val());
			/*// focus next input
			if(nextInput) {
				$("#" + nextInput).get(0).focus();
			}*/
		} else {
			$(this).addClass('clozeStyleIncorrect');
			wertiview.substantivesingular.updateClickLog(event, "cloze", 0, $(this).val(), $(this).data('wertiviewanswer'));
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
	},
	
	updateClickLog: function(event, extype, result, word, facit) {
	   var jQuery = wertiview.jQuery;
	   var contextDoc = event.data.context;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
		$.fn = $.prototype = jQuery.fn;

	   var nr_of_hits = $('span.wertiviewSubstantiveSingular').length;
	   wertiview.substantivesingular.totalresult += result;
	   wertiview.substantivesingular.totalclicked++;
	   var correctclicks = wertiview.substantivesingular.totalresult;
	   var totalclicks = wertiview.substantivesingular.totalclicked; 
	   var ratio1 =  correctclicks / totalclicks;
	   var ratio2 = correctclicks / nr_of_hits;
	   var dataString = "correct clicks / total clicks " + ratio1 + ", correct clicks / hits on the page " + ratio2;
	   jQuery.ajax({
            type: "POST",
            context: this,
            url: "http://localhost:8080/teaksta/WERTiServlet", // save data in a log file, without reloading the page
            contentType: "application/x-www-form-urlencoded;charset=UTF-8",
            data: {extype : extype, nr_of_exercises : nr_of_hits, word : word, facit : facit, correct : result, correctly_clicked : correctclicks, total_clicked : totalclicks, ratio1 : ratio1, ratio2 : ratio2},
            success: function() {
                //display message back to user here
                //alert("log " + dataString + " saved");
            }
        });
       return false;
	   //alert("correct clicks / total clicks" + ratio1);
	   //alert("correct clicks / hits on the page" + ratio2);
    }
 	};

