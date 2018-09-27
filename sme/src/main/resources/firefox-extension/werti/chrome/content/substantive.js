	wertiview.substantive = {

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
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.document); };
		$.fn = $.prototype = jQuery.fn;

		$('body').undelegate('span.wertiviewtoken', 'click', wertiview.substantive.clickHandler);
		$('body').undelegate('select.wertiviewinput', 'change', wertiview.substantive.clozeInputHandler);
		$('body').undelegate('span.wertiviewhint', 'click', wertiview.substantive.clozeHintHandler);
		$('body').undelegate('input.wertiviewinput', 'change', wertiview.substantive.clozeInputHandler);
		$('body').undelegate('input.wertiviewhint', 'click', wertiview.substantive.clozeHintHandler);  // was: span.wertiviewhint

		$('.wertiviewinput').each( function() {
			$(this).replaceWith($(this).data('wertiviewanswer'));
		});
		$('.wertiviewhint').remove();
	},

	colorize: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.document); };
		$.fn = $.prototype = jQuery.fn;

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
								if (children[k].classList.contains('wertiviewSubstantive')) {
									children[k].classList.add('colorizeStyleSubstantive');
								}
							}
	  				}
					}
				}
			}
		}

	},

	colorizeSpan: function(span, topic) {
		span.find('span.wertiviewSubstantive').addClass('colorizeStyleSubstantive');
	},

	click: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.document); };
		$.fn = $.prototype = jQuery.fn;

		var numExercises = 0;
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
								if (children[k].classList.contains('wertiviewSubstantive')) {
									numExercises += 1;
								}
							}
	  				}
					}
				}
			}
		}
		jQuery.ajax({
      type: "POST",
      url: "http://localhost:8080/teaksta/WERTiServlet", // save data (nr of generated exercises) in a log file, without reloading the page
      data: {nr_of_exercises : numExercises},
      success: function() {
      //display message back to user here
      }
    });

		// handle click
		$('body').delegate('span.wertiviewtoken', 'click', {context: contextDoc}, wertiview.substantive.clickHandler);
	},

	clickHandler: function(event) {
		var contextDoc = event.data.context;

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
			if (this.classList.contains('wertiviewSubstantive')) {  // was: wertiviewhit
				this.classList.add('clickStyleCorrect');
				wertiview.substantive.updateClickLog(event, "click", 1, $(this).text(), "");
			} else {
				this.classList.add('clickStyleIncorrect');
				wertiview.substantive.updateClickLog(event, "click", 0, $(this).text(), "");
			}
		}

		return false;
	},

	mc: function(contextDoc) {
		var jQuery = wertiview.jQuery;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.document); };
		$.fn = $.prototype = jQuery.fn;

		var tokens = [];
		wertiview.substantive.types = [];
		wertiview.substantive.hitList = [];

		var spanTags = $('span.wertiviewSubstantive');
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
		      wertiview.substantive.hitList.push($(this));
					tokens[$(this).text().toLowerCase()] = 1;
		  }});

		wertiview.substantive.maxLength = wertiview.substantive.MAX_MC;

		wertiview.activity.mc(contextDoc, wertiview.substantive.hitList,
				wertiview.substantive.clozeInputHandler,
				wertiview.substantive.clozeHintHandler,
				wertiview.substantive.mcGetOptions,
				wertiview.substantive.mcGetCorrectAnswer);

	},

	mcGetOptions: function($hit, capType){
		var options = [];
		var j = 0;
		// Get the list of distractors for the given hit (they are saved as a space-separated list in the attribute "distractors" of the wertiview span tag):
		wertiview.substantive.types = $hit.attr('distractors').split(" ");
		var correct_answer = $hit.attr('answer');
    wertiview.lib.shuffleList(wertiview.substantive.types);

    // Add the distractor forms to the options list:
    while (j < wertiview.substantive.types.length && options.length < wertiview.substantive.MAX_MC - 1) {
      // The forms that are homonymous to the correct form are excluded from the list of options:
      if (wertiview.substantive.types[j] != $hit.text().toLowerCase() && wertiview.substantive.types[j] != "") {
        var homonym = false;
        var k = 0;
        while (k < j) { //check for homonymes among the distractors
	        if (wertiview.substantive.types[k] == wertiview.substantive.types[j])
            homonym = true;
	        k++;
        }
        if (!homonym) options.push(wertiview.lib.matchCapitalization(wertiview.substantive.types[j], capType));
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

		var $hits = $('span.wertiviewSubstantive');
		var hitList = [];

		var spanTags = $('span.wertiviewSubstantive');
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
			wertiview.substantive.clozeInputHandler,
			wertiview.substantive.clozeHintHandler,
			wertiview.substantive.clozeGetCorrectAnswer,
			wertiview.substantive.clozeAddBaseform);
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
			//The following line was commented on the gtoahpa version but not on svn version
			//wertiview.substantive.updateClickLog(event, "cloze", 1, $(this).val(), $(this).val());
		} else {
				$(this).addClass('clozeStyleIncorrect');
				//The following line was commented on the gtoahpa version but not on svn version
				//wertiview.substantive.updateClickLog(event, "cloze", 0, $(this).val(), $(this).data('wertiviewanswer'));
			}
	},

	clozeHintHandler: function(event) {
		var jQuery = wertiview.jQuery;
		var contextDoc = event.data.context;
		  var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.document); };
		  $.fn = $.prototype = jQuery.fn;

		var nextInput;

		// fill in the answer by replacing input with text
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
	},

	updateClickLog: function(event, extype, result, word, facit) {
	  var jQuery = wertiview.jQuery;
	  var contextDoc = event.data.context;
		var $ = function(selector,context){ return new jQuery.fn.init(selector,contextDoc||window.document); };
		$.fn = $.prototype = jQuery.fn;

		var nr_of_hits = $('span.wertiviewSubstantive').length;
		wertiview.substantive.totalresult += result;
		wertiview.substantive.totalclicked++;
		var correctclicks = wertiview.substantive.totalresult;
		var totalclicks = wertiview.substantive.totalclicked;
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
		      }
		  });
		 return false;
    }
 	};
