# English: the reference. Every message the app shows has a value here; the
# North Sámi in se.ftl is shown beside it, or in its place, where it exists.
#
# Where a North Sámi line is shown with an English gloss, both are the same
# id. Where only English is shown, the id is here alone.

## Site chrome

choose-page = Choose a web page
choose-file = Choose a file
footer = Divvun · UiT The Arctic University of Norway

## Waiting on the backend

waiting-registry = Asking the backend which topics it offers
waiting-text = Asking the backend for the analysed text
waiting-upload = The backend is reading the text you sent

## Starting an exercise

start-heading = Start an exercise
start-lead = Any North Sámi page will do. Name one and the text is analysed once — every mode then plays over it.
start-lead-uploads = Any North Sámi page will do. Name one, or send a file of your own — either way the text is analysed once and every mode plays over it.
upload-lead = Send a page of your own and the text is analysed once, just as a page already on the web is — every mode then plays over it.
or = or
choose-file-to-use = Choose a file to use
file-format = The text must be an .html file.
file-limit = max 5 MB
choose-page-sub = Practise on a page already on the web
send = Send
uploads-unavailable = This deployment does not accept uploads. Practise on a page already on the web instead.

## Keeping an upload

keep-lead = The exercise keeps a link you can hand out
keep-yes = The file is kept, so the link works again later.
keep-no = The file is deleted overnight.

## An upload turned away

upload-no-file = No file was chosen
upload-too-large = The file is over the 5 MB limit
upload-not-a-page = The file is not an HTML page
upload-not-north-sami = Too little of the text reads as North Sámi

## A page that could not be read

page-failure-site-error = { $site } answered with an error ({ $status }). The site itself is failing, not teaksta: try again later, or choose another page.
page-failure-site-error-unnamed = The site answered with an error ({ $status }). The site itself is failing, not teaksta: try again later, or choose another page.
page-failure-unreachable = { $site } could not be reached, or did not answer in time. Check the address, or try again later.
page-failure-unreachable-unnamed = The text could not be read. Try again later.
page-failure-too-large = The page is too large to make an exercise from. Choose a shorter page.
page-failure-busy = teaksta is reading too many pages at once. Try again in a moment.
page-failure-bad-address = That is not a web address teaksta can read. Check it and try again.
page-failure-bad-scheme = Only web pages, starting http:// or https://, can be used.
page-failure-private-address = That address is on a private network, which teaksta cannot reach.
page-failure-not-served = That file is not one teaksta can read.
page-failure-no-such-text = The text this link pointed to is no longer stored.

## What went wrong otherwise

error-network = network error: { $detail }
error-status = backend answered { $status }
error-malformed = malformed response: { $detail }
error-incomplete = no topic, exercise mode or page was given
error-rejected = the text was turned away: { $reason }
error-unsupported = no fetch client outside the browser

## Topics and exercises

topic-heading = Topic
topic-count = { $picked } of { $total }
mode-heading = Exercise
mode-count = Same text, { $ways } ways in

# What each shipped topic marks, under its North Sámi name. The id carries
# the name the registry serves the topic under.
topic-Substantive = Nouns in any form
topic-SubstantiveSingular = Singular nouns only
topic-SubstantivePlural = Plural nouns only
topic-Subject = The subject of each clause
topic-Object = The object of each clause
topic-Adverbial = Adverbial phrases
topic-Conjunctions = Coordinating conjunctions
topic-NegVerbs = Negation verbs
topic-InfiniteVerbs = Non-finite verb forms
topic-VerbConjugation = Finite verbs and their conjugation

mode-colorize = Look at the coloured words.
mode-click = Click the right words!
mode-mc = Choose the right words!
mode-cloze = Write the right words!

# What each exercise asks for, in a few words.
mode-hint-colorize = Read & notice
mode-hint-click = Find them
mode-hint-mc = Pick a form
mode-hint-cloze = Type it

## The exercise

stage-foot = The same text is still loaded — switch mode to read it again.
mc-choose = Choose the missing form
mc-forms = Forms of the same word
cloze-write = Write the missing form
cloze-show-lemma = Show the base form

## Scoring

right = Right
highlighted = Highlighted words
see-result = See how it went
review-heading = Word by word
review-wrote = wrote

colorize-found = { $found ->
    [one] { $found } highlighted word form
   *[other] { $found } highlighted word forms
}
click-wrong = { $wrong ->
    [one] { $wrong } word picked that the topic does not mark
   *[other] { $wrong } words picked that the topic does not mark
}
click-result = { $right } of the { $wanted } marked words found
cloze-tries = { $tries ->
    [one] { $tries } form written so far
   *[other] { $tries } forms written so far
}
cloze-result = { $right } of the { $wanted } forms written right
mc-tries = { $tries ->
    [one] { $tries } form chosen so far
   *[other] { $tries } forms chosen so far
}
mc-result = { $settled } of the { $wanted } slots settled

## Ready-made texts

texts-heading = Ready-made texts
genre-story = story
genre-folktale = folktale
genre-factual = factual text
genre-textbook = textbook text
text-about = { $genre }, about { $words } words
text-about-tense = { $genre }, about { $words } words, { $tense }
tense-present = the verbs are in the present tense
tense-past = the verbs are in the past tense
