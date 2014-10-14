# VIEW ― a Platform for ICALL Systems

VIEW is a web platform for second language learners and programmers interested
in providing functionality for second language learning assistance. It is
capable of retrieving documents from web sites or from its own browser 
extension and then annotating their linguistic structure to help the learner. 
The web content can then be marked up in a modular way and the enhanced 
content exposed to the user through an interactive interface. This should 
ultimately allow the learner to understand the underlying principles of the 
target language, and thus acquire a firmer grasp of the language itself.

## Architecture
The system currently only supports English, German, and Spanish as target 
languages, but is highly modular and thus extensible towards other languages.

VIEW is confirmed to run on a Tomcat web server and relies on a recent JVM 
(version 1.6). It was only tested using the Sun JVM. The basic architecture 
relies on tomcat's servlets API and the (incubating) Apache UIMA API. Thus
it makes use of Java, AJAX, HTML and XML peripheral (e.g. Maven, Eclipse
EMF ...) technologies.

## Contact

The system is currently under development at the Seminar für 
Sprachwissenschaft (SfS) at the University of Tübingen, Germany. For more 
information, please refer to the documentation under `docs/` or contact the 
head of the VIEW development team, Prof. Detmar Meurers:
`dm@sfs.uni-tuebingen.de`

## Requirements
* A recent Tomcat (5 or higher, 6 recommended)
* Maven 2

## Instructions for Installation and Development

See `INSTALL.markdown` for instructions on how to install. See the 
documentation in `docs/`, especially `docs/internal-structure.markdown`, for 
some guidance if you want to work on the system.
