	wertiview.negverbs = {

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
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.document); };
		$.fn = $.prototype = jQuery.fn;

		$('body').undelegate('span.wertiviewtoken', 'click', wertiview.negverbs.clickHandler);
		$('body').undelegate('select.wertiviewinput', 'change', wertiview.negverbs.clozeInputHandler);
		$('body').undelegate('span.wertiviewhint', 'click', wertiview.negverbs.clozeHintHandler);
		$('body').undelegate('input.wertiviewinput', 'change', wertiview.negverbs.clozeInputHandler);
		$('body').undelegate('input.wertiviewhint', 'click', wertiview.negverbs.clozeHintHandler);  // was: span.wertiviewhint

		$('.wertiviewinput').each( function() {
			$(this).replaceWith($(this).data('wertiviewanswer'));
		});
		$('.wertiviewhint').remove();
	},

	colorize: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.document); };
		$.fn = $.prototype = jQuery.fn;

		// check if the span is in a menu link or in a widget, change remaining wertiviewtoken spans to mouseover pointer
		var spanTags = document.getElementsByClassName('wertiview');
		for (i = 0; i < spanTags.length; i++) {
			if (spanTags[i].parentElement.classList) {
				var patt_widget = /widget/g;
				var res_widget = patt_widget.test(spanTags[i].parentElement.classList);
				if (spanTags[i].parentElement.tagName != 'A' && !res_widget)  {
					if (spanTags[i].hasChildNodes()) {
						var children = spanTags[i].childNodes;
						for (var k = 0; k < children.length; k++) {
							if (children[k].classList) {
								if (children[k].classList.contains('wertiviewConNeg')) {
									children[k].classList.add('colorizeStyleNegVerbs');
								}
							}
						}
					}
				}
			}
		}
	},

	colorizeSpan: function(span, topic) {
		span.find('span.wertiviewConNeg').addClass('colorizeStyleNegVerbs');
	},

	click: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.document); };
		$.fn = $.prototype = jQuery.fn;

		// check if the span is in a menu link or in a widget, change remaining wertiviewtoken spans to mouseover pointer
		var spanTags = document.getElementsByClassName('wertiview');
		for (i = 0; i < spanTags.length; i++) {
			if (spanTags[i].parentElement.classList) {
				var patt_widget = /widget/g;
				var res_widget = patt_widget.test(spanTags[i].parentElement.classList);
				if (spanTags[i].parentElement.tagName != 'A' && !res_widget)  {
					if (spanTags[i].hasChildNodes()) {
						var children = spanTags[i].childNodes;
						for (var k = 0; k < children.length; k++) {
							if (children[k].classList) {
								children[k].style.cursor = "pointer"; //({'cursor': 'pointer'});
							}
						}
					}
				}
			}
		}

		// handle click
		$('body').delegate('span.wertiviewtoken', 'click', {context: contextDoc}, wertiview.negverbs.clickHandler);
	},

	clickHandler: function(event) {
		var contextDoc = event.data.context;

		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.document); };
		$.fn = $.prototype = jQuery.fn;

		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.document); };
		$.fn = $.prototype = jQuery.fn;

		//check if parent element is a link, a menu item or a widget
		var parent_elements = [];
		var current_element = this;
		while (current_element) {
			parent_elements.unshift(current_element);
			current_element = current_element.parentNode;
		}
		var widget_found = false;
		var menu_found = false;
		var a_found = false;

		for (i = 0; i < parent_elements.length; i++) {
			var patt_widget = /widget/g;
			var res_widget = patt_widget.test(parent_elements[i].classList);
			var patt_menu = /menu/g;
			var res_menu = patt_menu.test(parent_elements[i].classList);
			if (res_widget) {
				widget_found = true;
			}
			if (res_menu) {
				menu_found = true;
			}
			if (parent_elements[i].tagName == 'A') {
				a_found = true;
			}
		}

		if (!widget_found && !menu_found && !a_found) {
			if (this.classList.contains('wertiviewConNeg')) {  // was: wertiviewhit
				this.classList.add('clickStyleCorrect');
			} else {
				this.classList.add('clickStyleIncorrect');
			}
		}
    return false;
	},

	mc: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.document); };
		$.fn = $.prototype = jQuery.fn;

		var tokens = [];
		wertiview.negverbs.types = [];  // answer options that will be displayed  as a drop-down list
		wertiview.negverbs.hitList = [];  // words that will be turned to exercises

		var spanTags = $('span.wertiviewConNeg');
		//check if parent element is a link, a menu item or a widget
		spanTags.each(function(){
			var parent_elements = [];
			var current_element = $(this)[0];
			while (current_element) {
				parent_elements.unshift(current_element);
				current_element = current_element.parentNode;
			}
			var widget_found = false;
			var menu_found = false;
			var a_found = false;

			for (i = 0; i < parent_elements.length; i++) {
				var patt_widget = /widget/g;
				var res_widget = patt_widget.test(parent_elements[i].classList);
				var patt_menu = /menu/g;
				var res_menu = patt_menu.test(parent_elements[i].classList);
				if (res_widget) {
					widget_found = true;
				}
				if (res_menu) {
					menu_found = true;
				}
				if (parent_elements[i].tagName == 'A') {
					a_found = true;
				}
			}
			if (!widget_found && !menu_found && !a_found) {
					wertiview.negverbs.hitList.push($(this));
					tokens[$(this).text().toLowerCase()] = 1;
			}});

		wertiview.negverbs.maxLength = wertiview.negverbs.MAX_MC;

		wertiview.activity.mc(contextDoc, wertiview.negverbs.hitList,
				wertiview.negverbs.clozeInputHandler,
				wertiview.negverbs.clozeHintHandler,
				wertiview.negverbs.mcGetOptions,
				wertiview.negverbs.mcGetCorrectAnswer);

	},

	mcGetOptions: function($hit, capType){
		var options = [];
		var j = 0;
		// Get the list of distractors for the given hit (they are saved as a space-separated list in the attribute "distractors" of the wertiview span tag):
		wertiview.negverbs.types = $hit.attr('distractors').split(" ");
		var correct_answer = $hit.attr('answer');
    wertiview.lib.shuffleList(wertiview.negverbs.types);

    // Add the distractor forms to the options list:
    while (options.length < wertiview.negverbs.maxLength - 1) {
      // The forms that are homonymous to the correct form are excluded from the list of options:
      if (wertiview.negverbs.types[j] != $hit.text().toLowerCase() && wertiview.negverbs.types[j] != "") {
      	options.push(wertiview.lib.matchCapitalization(wertiview.negverbs.types[j], capType));
      }
			j++;
		}

		options.push(wertiview.lib.matchCapitalization(correct_answer, capType));

		wertiview.lib.shuffleList(options);
		return options;
	},

	mcGetCorrectAnswer: function($hit, capType){
		var correct_answer = $hit.attr('answer');
		return correct_answer;
	},

	clozeGetCorrectAnswer: function($hit, capType){
		return $hit.attr('possibleforms');
	},

	cloze: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.document); };
		$.fn = $.prototype = jQuery.fn;

		var hitList = [];
		var spanTags = $('span.wertiviewConNeg');
		//check if parent element is a link, a menu item or a widget
		spanTags.each(function(){
			var parent_elements = [];
			var current_element = $(this)[0];
			while (current_element) {
					parent_elements.unshift(current_element);
					current_element = current_element.parentNode;
			}
			var widget_found = false;
			var menu_found = false;
			var a_found = false;

			for (i = 0; i < parent_elements.length; i++) {
				var patt_widget = /widget/g;
				var res_widget = patt_widget.test(parent_elements[i].classList);
				var patt_menu = /menu/g;
				var res_menu = patt_menu.test(parent_elements[i].classList);
				if (res_widget) {
					widget_found = true;
				}
				if (res_menu) {
					menu_found = true;
				}
				if (parent_elements[i].tagName == 'A') {
					a_found = true;
				}
			}
			if (!widget_found && !menu_found && !a_found) {
					hitList.push($(this));
			}
		});

		wertiview.activity.cloze(contextDoc, hitList,
				wertiview.negverbs.clozeInputHandler,
				wertiview.negverbs.clozeHintHandler,
				wertiview.negverbs.clozeGetCorrectAnswer,
				wertiview.negverbs.clozeAddBaseform);
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
		  var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.document); };
		  $.fn = $.prototype = jQuery.fn;

		var nextInput;

		//check if the user input match one of the possible answers
		var answer_split = $(this).data('wertiviewanswer').toLowerCase().split(" ");
		var correct_answer = false;
		for (i = 0; i < answer_split.length; i++) {
			if ($(this).val().toLowerCase() == answer_split[i]) {
				correct_answer = true;
			}
		}
		// if the answer is correct, turn into text, else color text within input
		if(correct_answer) {
			$text = $("<span>");
			$text.addClass('wertiview');
			$text.addClass('clozeStyleCorrect');
			$text.text($(this).val().toLowerCase());
			if($(this).data('wertiviewnexthit')) {
				nextInput = $(this).data('wertiviewnexthit');
			}
			wertiview.lib.replaceInput($(this).parent(), $text);
		} else {
				$(this).addClass('clozeStyleIncorrect');
			}
	},

	clozeHintHandler: function(event) {
		var jQuery = wertiview.jQuery;
		var contextDoc = event.data.context;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.document); };
		$.fn = $.prototype = jQuery.fn;

		var nextInput;

		//fill in the answer by replacing input with text where each answer is separated by /
		var answer_sep = $(this).prev().data('wertiviewanswer').replace(/\s/g, "/");
		$text = $("<span>");
		$text.addClass('wertiview');
		$text.addClass('clozeStyleProvided');
		$text.text(answer_sep);
		if($(this).prev().data('wertiviewnexthit')) {
			nextInput = $(this).prev().data('wertiviewnexthit');
		}
		wertiview.lib.replaceInput($(this).parent(), $text);

		return false;
	}
	};
