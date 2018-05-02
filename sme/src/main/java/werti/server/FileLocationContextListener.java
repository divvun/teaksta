package werti.server;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.PrintWriter;
import java.io.OutputStreamWriter;
import java.io.FileOutputStream;
import java.io.FileWriter;
import java.net.URL;
import java.util.Arrays;
import java.util.Enumeration;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.Locale;

import javax.servlet.RequestDispatcher;
import javax.servlet.ServletConfig;
import javax.servlet.ServletException;
import javax.servlet.http.HttpServlet;
import javax.servlet.http.HttpServletRequest;
import javax.servlet.http.HttpServletResponse;

import javax.servlet.ServletContext;
import javax.servlet.ServletContextEvent;
import javax.servlet.ServletContextListener;

import org.apache.commons.lang.StringEscapeUtils;
import org.apache.log4j.Logger;
import org.apache.uima.jcas.JCas;
import org.jsoup.Jsoup;
import org.jsoup.nodes.Node;
import org.jsoup.nodes.Document;
import org.openid4java.OpenIDException;
import org.openid4java.consumer.ConsumerException;
import org.openid4java.consumer.ConsumerManager;
import org.openid4java.discovery.DiscoveryException;
import org.openid4java.discovery.DiscoveryInformation;
import org.openid4java.discovery.Identifier;
import org.openid4java.message.AuthRequest;
import org.openid4java.message.MessageException;

import werti.WERTiContext;
import werti.WERTiContext.WERTiContextException;
import werti.util.ActivitiesSessionLoader;
import werti.util.HTMLEnhancer;
import werti.util.HTMLUtils;
import werti.util.JSONEnhancer;
import werti.util.PageHandler;
import werti.util.PostRequest;
import werti.util.PracticeHandler;

import com.google.gson.Gson;

import weka.core.Instances;
import weka.filters.Filter;

import werti.util.Constants;

import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.util.Iterator;
import javax.servlet.ServletContext;
import javax.servlet.ServletOutputStream;

import org.apache.commons.fileupload.FileItem;
import org.apache.commons.fileupload.FileUploadException;
import org.apache.commons.fileupload.disk.DiskFileItemFactory;
import org.apache.commons.fileupload.servlet.ServletFileUpload;
import org.apache.commons.fileupload.FileUploadBase;

import org.apache.commons.lang3.RandomStringUtils;


/*
@WebListener*/
public class FileLocationContextListener implements ServletContextListener {

    public void contextInitialized(ServletContextEvent servletContextEvent) {
    	ServletContext ctx = servletContextEvent.getServletContext();

      String rnd_name = RandomStringUtils.randomAlphanumeric(10);
      System.out.println("rnd_name="+rnd_name);

      // Directory for files upload
      String relativePath = ctx.getInitParameter("files_dir");
      //File file = new File(rootPath + File.separator + relativePath);
    	File file = new File(relativePath);
    	if(!file.exists()) file.mkdirs();
    	System.out.println("File Directory created to be used for storing files");
    	ctx.setAttribute("FILES_DIR_FILE", file);
      //ctx.setAttribute("FILES_DIR", rootPath + File.separator + relativePath);
    	ctx.setAttribute("FILES_DIR", relativePath);

      // Directory for files we have permission to store
      String relativePathPrm = ctx.getInitParameter("files_prm_dir");
    	File filePrm = new File(relativePathPrm);
    	if(!filePrm.exists()) filePrm.mkdirs();
    	ctx.setAttribute("FILES_PRM_DIR_FILE", filePrm);
    	ctx.setAttribute("FILES_PRM_DIR", relativePathPrm);

      // Directory for files we will delete
      String relativePathTmp = ctx.getInitParameter("files_tmp_dir");
    	File fileTmp = new File(relativePathTmp);
    	if(!fileTmp.exists()) fileTmp.mkdirs();
    	ctx.setAttribute("FILES_TMP_DIR_FILE", fileTmp);
    	ctx.setAttribute("FILES_TMP_DIR", relativePathTmp);

    }

	public void contextDestroyed(ServletContextEvent servletContextEvent) {
		//do cleanup if needed
	}

}
