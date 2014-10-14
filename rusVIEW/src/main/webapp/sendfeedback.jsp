   	<%@ page import="net.tanesha.recaptcha.ReCaptchaImpl" %>
   	<%@ page import="net.tanesha.recaptcha.ReCaptchaResponse" %>
	<%@ page import="java.util.*" %>
	<%@ page import="javax.mail.*" %>
	<%@ page import="javax.mail.internet.*" %>

<html>
<body>
<%
       String remoteAddr = request.getRemoteAddr();
       ReCaptchaImpl reCaptcha = new ReCaptchaImpl();
       reCaptcha.setPrivateKey("6LcoXLsSAAAAANcyuPl2aUpkQP0ZwRoDMvvjci5D");

       String challenge = request.getParameter("recaptcha_challenge_field");
       String uresponse = request.getParameter("recaptcha_response_field");
       ReCaptchaResponse reCaptchaResponse = reCaptcha.checkAnswer(remoteAddr, challenge, uresponse);

       if (reCaptchaResponse.isValid()) {
         out.print("Answer was entered correctly!");

	String host = "smtpserv.uni-tuebingen.de";
	String to = "adriane@sfs.uni-tuebingen.de";
	String from = request.getParameter("from");
	String subject = "VIEW feedback";
	String messageText = request.getParameter("url") + "\n\n" + request.getParameter("message");
	
	Properties props = System.getProperties();
	props.put("mail.host", host);
	props.put("mail.transport.protocol", "smtp");
	
	Session mailSession = Session.getDefaultInstance(props, null);
	mailSession.setDebug(false);
 
	Message msg = new MimeMessage(mailSession);
	msg.setFrom(new InternetAddress(from));
	InternetAddress[] address = {new InternetAddress(to)};
	msg.setRecipients(Message.RecipientType.TO, address);
	msg.setSubject(subject);
	msg.setSentDate(new Date());
	msg.setText(messageText);
 
	Transport.send(msg);
	
	response.sendRedirect("index.jsp?content=thanks");

	} else {
		out.print("Sorry, verification failed.  Please press back to go back to the form and try again.");
	}
       
     %>
</body>
</html>
