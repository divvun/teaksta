wertiview.ns(function() {
	wertiview.konjunktiv = {	

	// for documentation of these variables, see pos.js
	MAX_MC: 5,
	maxLength: 5,

	types: [],
	
	remove: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
		$.fn = $.prototype = jQuery.fn;

		$('body').undelegate('span.wertiviewtoken', 'click', wertiview.konjunktiv.clickHandler);
		$('body').undelegate('select.wertiviewinput', 'change', wertiview.konjunktiv.clozeInputHandler);
		$('body').undelegate('span.wertiviewhint', 'click', wertiview.konjunktiv.clozeHintHandler);
		$('body').undelegate('input.wertiviewinput', 'change', wertiview.konjunktiv.clozeInputHandler);
		$('body').undelegate('input.wertiviewhint', 'click', wertiview.konjunktiv.clozeHintHandler);

		$('.wertiviewanswered').each( function() {
			$(this).replaceWith($(this).data('wertivieworiginaltext'));
		});
		
		$('.wertiviewinput').each( function() {
			$(this).parent().replaceWith($(this).data('wertivieworiginaltext'));
		});
		
		$('span.wertiviewbaseform').remove();
		$('.wertiviewhint').remove();
	},

	colorize: function(contextDoc) {
		var jQuery = wertiview.jQuery;
        var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
        $.fn = $.prototype = jQuery.fn;        
        
		$('span.wertiviewkonjauxwuerden').addClass('colorizeStyleKonjunktivWuerden');
		$('span.wertiviewkonjauxhaben').addClass('colorizeStyleKonjunktivHaben');
		$('span.wertiviewkonjmain').addClass('colorizeStyleKonjunktivMain');
	},
	
	colorizeSpan: function(span, topic, index) {
		span.find('span.wertiviewkonjauxwuerden').addClass('colorizeStyleKonjunktivWuerden');
		span.find('span.wertiviewkonjauxhaben').addClass('colorizeStyleKonjunktivHaben');
		span.find('span.wertiviewkonjmain').addClass('colorizeStyleKonjunktivMain');
	},
	
	click: function(contextDoc) {
		var jQuery = wertiview.jQuery;
                var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
                $.fn = $.prototype = jQuery.fn;

		// change all wertiviewtoken spans to mouseover pointer
		$('span.wertiviewtoken').css({'cursor': 'pointer'});

		// handle POS clicking
		$('body').delegate('span.wertiviewtoken', 'click', {context: contextDoc}, wertiview.konjunktiv.clickHandler);
	},

	clickHandler: function(event) {
		var jQuery = wertiview.jQuery;
		var contextDoc = event.data.context;
        var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
        $.fn = $.prototype = jQuery.fn;
                
		if($(this).hasClass('wertiviewkonjfin')) {
			$(this).addClass('clickStyleCorrect');
		} else {
			$(this).addClass('clickStyleIncorrect');
		} 
		return false;
	},
	
	/*mc: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
        $.fn = $.prototype = jQuery.fn;

		// get potential spans
		var $hits = $('span.wertiviewkonjfin');
		
		var hitList = [];
		$hits.each( function() {
			verbforms = $(this).attr('title').split(';');
			// quick and dirty check for whether lemmatizer worked
			// and lemma is actually an infinitive
			if (verbforms[0].slice(-1) == "n") {
				hitList.push($(this));
			}
		});
		
		wertiview.activity.mc(contextDoc, hitList, 
				wertiview.konjunktiv.mcInputHandler, 
				wertiview.konjunktiv.mcHintHandler, 
				wertiview.konjunktiv.mcGetOptions, 
				wertiview.konjunktiv.mcGetCorrectAnswer, 
				wertiview.konjunktiv.mcColor, false, 1);
	},
	
	mcGetOptions: function($hit, capType){
		var types = ['regular', 'irregular'];
		return types;
	},
	
	mcGetCorrectAnswer: function($hit, capType){
		if ($hit.hasClass('wertiviewkonjreg')) {
			return 'regular';
		}
		else {
			return 'irregular';
		}
	},
	
	mcColor: function($hit, capType) {
		verbforms = $hit.attr('title').split(';');
		$hit.text(verbforms[0]);
		$hit.append(' ');		
		$hit.addClass('mcStyleHighlight');
	},
	
	mcInputHandler: function(event) {
		var contextDoc = event.data.context;
		
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
		$.fn = $.prototype = jQuery.fn;

		// if the answer is correct, turn into text, else color text within input
		if($(this).val().toLowerCase() == $(this).data('wertiviewanswer').toLowerCase()) {
			$text = $('<span>');
			$text.addClass('wertiviewanswered');
			$text.data('wertivieworiginaltext', $(this).data('wertivieworiginaltext'));
			$text.addClass('clozeStyleCorrect');
			$text.text($(this).data('wertivieworiginaltext'));
			$answer = $('<span>');
			$answer.addClass('mcStyleAnswer');
			$answer.text(' (' + $(this).data('wertiviewanswer').toLowerCase() + ')');
			$text.append($answer);
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
		var contextDoc = event.data.context;
		
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
		$.fn = $.prototype = jQuery.fn;

		// fill in the answer by replacing input with text
		$text = $("<span>");
		$text.addClass('wertiviewanswered');
		$text.data('wertivieworiginaltext', $(this).prev().data('wertivieworiginaltext'));
		$text.addClass('clozeStyleProvided');
		$text.text($(this).prev().data('wertivieworiginaltext'));
		$answer = $('<span>');
		$answer.addClass('mcStyleAnswer');
		$answer.text(' (' + $(this).prev().data('wertiviewanswer').toLowerCase() + ')');
		$text.append($answer);
		if($(this).prev().data('wertiviewnexthit')) {
			nextInput = $(this).prev().data('wertiviewnexthit');
		}
		wertiview.lib.replaceInput($(this).parent(), $text);
		
		return false;
	},*/
	
	cloze: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.content.document); };
        $.fn = $.prototype = jQuery.fn;

		// get potential spans
		var $hits = $('span.wertiviewkonjfin');
		
		var hitList = [];
		$hits.each( function() {
			verbforms = $(this).attr('title').split(';');
			// quick and dirty check for whether lemmatizer worked
			// and lemma is actually an infinitive
			if (verbforms[0].slice(-1) == "n") {
				hitList.push($(this));
			}
		});

		wertiview.activity.cloze(contextDoc, hitList, 
				wertiview.konjunktiv.clozeInputHandler, 
				wertiview.konjunktiv.clozeHintHandler, 
				wertiview.konjunktiv.clozeGetCorrectAnswer,
				wertiview.konjunktiv.clozeAddBaseform);
	},
	
	clozeAddBaseform: function($hit, capType, $){
		// create baseform info
		var $baseform = $('<span>');
		$baseform.addClass('clozeStyleBaseform');
		$baseform.addClass('wertiviewbaseform');
		var verbforms = $hit.attr('title').split(';');
		$baseform.text(' (' + verbforms[0] + ')');
		$hit.append($baseform);	
	},
	
	clozeGetCorrectAnswer: function($hit, capType){
		return $hit.text();
	},

	clozeInputHandler: function(event) {
		var contextDoc = event.data.context;
		
		var jQuery = wertiview.jQuery;
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
		
		return false;
	},

	clozeHintHandler: function(event) {
		var contextDoc = event.data.context;
		
		var jQuery = wertiview.jQuery;
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
		if(nextInput && $("#" + nextInput).length == 1) {
			$("#" + nextInput).get(0).focus();
		}*/
		
		return false;
	}
	};
}); // REMOVE-WITH-MAVEN-REPLACER-PLUGIN
