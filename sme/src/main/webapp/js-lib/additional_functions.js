function setUrl_and_submit(anchor) {
		      var address=anchor.innerHTML;
		      var settingsform=parent.document.getElementsByName("activityForm");
		      var url=parent.document.getElementsByName("url");
		      url[0].value=address;
		      settingsform[0].submit();
} 