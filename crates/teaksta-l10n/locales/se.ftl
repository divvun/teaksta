# North Sámi.
#
# Every message here has an English value of the same id in en.ftl, which is
# the reference. A message with no line here is shown in English alone, so a
# translation is added by adding its id to this file and nothing else.
#
# Each line below is the wording the app already showed, most of it carried
# over from the webapp teaksta replaces.

-waiting = Vuorddát…

## Site chrome

choose-page = Vállje neahttasiiddu
choose-file = Vállje fiilla
footer = Divvun · UiT Norgga árktalaš universitehta

## Waiting on the backend

waiting-registry = { -waiting }
waiting-text = { -waiting }
waiting-upload = { -waiting }

## Starting an exercise

or = dahje
choose-file-to-use = Vállje fiilla maid háliidat geavahit
file-format = Teaksta ferte leat .html formáhtas.
send = Sádde

## Keeping an upload

keep-lead = Sáhtát addit hárjehusa liŋkka ohppiide. Vállje nuppi molssaeavttu:
keep-yes = Fiila vurkejuvvo nu ahte sáhtát geavahit liŋkka ođđasit eará háve.
keep-no = Fiila sihkkojuvvo gaskaija áigge.

## An upload turned away

upload-no-file = Vajálduhttet sáddet fiilla!
upload-too-large = Fiila lea menddo stuoris! Lobálaš sturrodat: 5MB.
upload-not-a-page = Fiilla formáhta ii leat html! Lobálaš formáhta: html.
upload-not-north-sami = Fiila ii sisttisdoala davvisámegiela!

## Topics and exercises

topic-heading = Fáddá
mode-heading = Hárjehus

# The instruction each exercise gives. The server sends these as the modes'
# labels, so they are what every client shows.
mode-colorize = Geahča ivdnejuvvon sániid.
mode-click = Coahkkal rivttes sániid!
mode-mc = Vállje rivttes sániid!
mode-cloze = Čále rivttes sániid!

## Scoring

right = Rivttes
highlighted = Ivdnejuvvon sánit
see-result = Geahča bohtosa

## Ready-made texts
#
# The heading and the descriptions are the webapp's own: it headed the section
# its lists stood in "Materiálat", and described each text as
# "(muitalus, su. 190 sáni, vearbbat leat preseanssas)".

texts-heading = Materiálat
genre-story = muitalus
genre-folktale = máinnas
genre-factual = áššeteaksta
genre-textbook = oahppogirjeteaksta
text-about = { $genre }, su. { $words } sáni
text-about-tense = { $genre }, su. { $words } sáni, { $tense }
tense-present = vearbbat leat preseanssas
tense-past = vearbbat leat preterihtas
