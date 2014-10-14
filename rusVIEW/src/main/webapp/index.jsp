<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">

<%@page import="java.io.File"%>
<%@page import="java.net.URLEncoder"%>
<%@page import="werti.server.Activities"%>
<%@page import="werti.util.ActivitiesSessionLoader"%>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
		<meta http-equiv="content-type" content="text/html; charset=UTF-8"/>
        <%
        // prevent search engines from following the example enhancement links
        String content = request.getParameter("content");
        if ("activity-help".equals(content)) {
        %>
            <meta name="robots" content="nofollow" />
        <% 
        }
        %>
		<link type="text/css" href="/VIEW/view.css" rel="stylesheet" />
		<title>Welcome to VIEW</title>
</head>
<body>
    <div id="wrapper">


      <!-- Logo, Bilder -->

      <div id="banner">
<table width="910" border="0" cellspacing="0" cellpadding="0">
  <tr>
    <td style="width:210px;" align="center" valign="top">
      <a href="http://www.uni-tuebingen.de" rel="external">
        <img src="/VIEW/images/ut_logo_brosch_p280c-8-OK.gif" alt="Universität Tübingen" width="182"
          style="border:0px; margin-top: 29px;"/>
      </a>
    </td>
    <td class="center-column" style="width: 512px;" valign="top">
      <img src="/VIEW/images/wurmlingerkapelle.jpg" alt="VIEW "/><!-- width="513" height="128" -->
    </td>
    <td style="width:188px;" align="center" valign="top">
      <a href="http://www.sfs.uni-tuebingen.de" rel="external">
        <img style="border: 0px; margin-top: 19px;" src="/VIEW/images/sfs-rgb-OK.gif" alt="Seminar für Sprachwissenschaft" width="182"/>
      </a>
    </td>
  </tr>
</table>
      </div>
      <div id="name">Visual Input Enhancement of the Web (VIEW)</div>
      <div id="main">
        <div id="menulinks">
        	<ul>
			    <li><a href="/VIEW/index.jsp?content=home">Home</a></li>
				<li><a href="/VIEW/index.jsp?content=about">About VIEW</a></li>
				<li><a href="/VIEW/index.jsp?content=intro">Getting Started</a></li>
				<li><a href="/VIEW/index.jsp?content=activities">Topics and Activities</a>
					<ul>
						<%
						
						// load activities into/from session
						Activities acts = ActivitiesSessionLoader.createActivitiesInSession(request);

						for (String basename : acts) {
							String displayName = acts.getActivity(basename).getName();
							if (acts.getActivity(basename).isEnabled()) {
								out.println("<li><a href=\"/VIEW/index.jsp?content=activity-help&amp;activity=" + URLEncoder.encode(basename, "UTF-8")  + "\">" + displayName + "</a></li>");
							}
						}
						%>
					</ul>
				</li>
				<li><a href="/VIEW/index.jsp?content=firefox-extension">Firefox Extension</a></li>
				<li><a href="/VIEW/index.jsp?content=changelog">Changelog</a></li>
				<li><a href="/VIEW/index.jsp?content=feedback">Feedback</a></li>
			</ul>          
        </div>
        <div id="mainpartwrapper">
          <div id="mainpart">
			<%
			if (content == null) {
				content = "tmpl/home.jsp";
			} else {
				content = "tmpl/" + content + ".jsp";
			}
			%>
			<jsp:include page="<%= content %>" />
          </div>
        </div>
        <!-- end of main -->

      </div>
      <div id="footer"></div>
</div>
</body>
</html>
