
	wertiview.rusverbimperfective = {

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

		$('body').undelegate('span.wertiviewtoken', 'click', wertiview.rusverbimperfective.clickHandler);
		$('body').undelegate('select.wertiviewinput', 'change', wertiview.rusverbimperfective.clozeInputHandler);
		$('body').undelegate('span.wertiviewhint', 'click', wertiview.rusverbimperfective.clozeHintHandler);
		$('body').undelegate('input.wertiviewinput', 'change', wertiview.rusverbimperfective.clozeInputHandler);
		$('body').undelegate('input.wertiviewhint', 'click', wertiview.rusverbimperfective.clozeHintHandler);  // was: span.wertiviewhint
		
		$('.wertiviewinput').each( function() {
			$(this).replaceWith($(this).data('wertiviewanswer'));
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

		$('span.wertiviewhit').addClass('colorizeStyleRusVerbImperfective');
	},
	
	colorizeSpan: function(span, topic, index) {
span.find('span.wertiviewhit').addClass('colorizeStyleRusVerbImperfective');
	},

	click: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ 
			return new jQuery.fn.init(selector,contextDoc||window.content.document); 
			};
		$.fn = $.prototype = jQuery.fn;

		// change all wertiviewtoken spans to mouseover pointer
		$('span.wertiviewtoken').css({'cursor': 'pointer'}); 
		
		// handle click
		$('body').delegate('span.wertiviewtoken', 'click', {context: contextDoc}, wertiview.rusverbimperfective.clickHandler); 
	},

	clickHandler: function(event) {
		var contextDoc = event.data.context;

		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ 
			return new jQuery.fn.init(selector,contextDoc||window.content.document); 
			};
		$.fn = $.prototype = jQuery.fn;
		
		if($(this).hasClass('wertiviewhit')) {  // was: wertiviewhit
			$(this).addClass('clickStyleCorrect');
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
		wertiview.rusverbimperfective.types = [];
		//alert($hits.length+" hits");
		$hits.each( function() {
			hitList.push($(this));
			//alert($(this).text());
			tokens[$(this).text().toLowerCase()] = 1;
		});

		wertiview.rusverbimperfective.maxLength = wertiview.rusverbimperfective.MAX_MC;
		wertiview.activity.mc(contextDoc, hitList, 
				wertiview.rusverbimperfective.clozeInputHandler, 
				wertiview.rusverbimperfective.clozeHintHandler, 
				wertiview.rusverbimperfective.mcGetOptions, 
				wertiview.rusverbimperfective.mcGetCorrectAnswer);

	},
	
	mcGetOptions: function($hit, capType){
		var options = [];
		var j = 0;
		// Get the list of distractors for the given hit (they are saved as a space-separated list in the attribute "distractors" of the wertiview span tag):
		wertiview.rusverbimperfective.types = $hit.attr('distractors').split(" ");
	    wertiview.lib.shuffleList(wertiview.rusverbimperfective.types);
        
        // Add the distractor forms to the options list:
        while (j < wertiview.rusverbimperfective.types.length && options.length < wertiview.rusverbimperfective.MAX_MC - 1) {
            // The forms that are homonymous to the correct form are excluded from the list of options:
            if (wertiview.rusverbimperfective.types[j].toLowerCase() != $hit.text().toLowerCase() && wertiview.rusverbimperfective.types[j] != "") 
            {
               options.push(wertiview.lib.matchCapitalization(wertiview.rusverbimperfective.types[j], capType)); 
            }
			j++;
		}
		
		options.push(wertiview.lib.matchCapitalization($hit.text(), capType));
		wertiview.lib.shuffleList(options);
		return options;
	},
	
	mcGetCorrectAnswer: function($hit, capType){
		return $hit.text();
	},
	
	cloze: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ 
			return new jQuery.fn.init(selector,contextDoc||window.content.document); 
			};
		$.fn = $.prototype = jQuery.fn;
		
		// get potential spans
		var $hits = $('span.wertiviewhit');

		var hitList = [];
		$hits.each( function() {
			hitList.push($(this));				
		}); 
		
		wertiview.activity.cloze(contextDoc, hitList, 
				wertiview.rusverbimperfective.clozeInputHandler, 
				wertiview.rusverbimperfective.clozeHintHandler, 
				wertiview.rusverbimperfective.mcGetCorrectAnswer,
				wertiview.rusverbimperfective.clozeAddBaseform);
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
			if($(this).data('wertiviewnexthit')) {   
				nextInput = $(this).data('wertiviewnexthit');
			}
			wertiview.lib.replaceInput($(this).parent(), $text);

		} else {
			$(this).addClass('clozeStyleIncorrect');
		}
		return false;
	},

	clozeHintHandler: function(event) {
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
