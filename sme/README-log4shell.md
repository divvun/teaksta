**Konteaksta** is an ICALL tool used to work with authentic North Saami texts in education. Students or teachers could have the tool remove certain kinds of words from texts and ask the student to fill them back in in the correct grammatical form, either as a form of multiple-choice. 

It was forked from [WERTi](http://sifnos.sfs.uni-tuebingen.de/WERTi/), developed at the University of Tübingen, and adapted for North Saami at [Giellatekno](https://giellatekno.uit.no). The last working version ran on Tomcat 7 and Java 7 and was hosted on the gtoahpa server.

Konteaksta was taken offline in 2021 due to the [Log4shell](https://en.wikipedia.org/wiki/Log4Shell) vulnerability in log4j, used for logging in many Java applications including Konteaksta. To get it back online, upgrading log4j to a secure version was needed, which proved a bit troublesome because the programmer in charge had not been developing in Java before. 

Simply upgrading the version of log4j 2.x was simple enough, but in addition to this, there were two related problems:
- Konteaksta also used log4j 1.x, and this would have to be ported to log4j 2.x
- Maltparser, used by WERTi to parse English, but unused (commented out) in Konteaksta, used an old version of log4j, and was no longer being updated.

The job to be done was thus to change the log4j 1.x calls to log4j 2.x calls, and remove Maltparser entirely from the code to be entirely sure it was not in use. The work then done is documented more closely in [log4shell-debug-log.txt](log4shell-debug-log.txt). Basically, the porting to log4j 2.x seemed to work fine in local testing, but when having removed Maltparser from the pom.xml and deleted the commented-out lines of code, Konteaksta did no longer start in Tomcat, even when the change was reversed.

The main problem when trying to debug this was that the programmer (hi, I'm writing this readme) was not familiar with Java, Maven or Tomcat and had trouble finding his way around, seeing what might be the problem, where to find error logs and so forth. Hopefully, someone more familiar with Java could try running the code, seeing what causes it to crash and fix that. If the code runs using only log4j 2.x and no Maltparser, we should be good.

Of course, the Java and Tomcat versions used are very outdated and should also be upgraded. Hopefully a Java programmer would know if that entails much work in the code or not.