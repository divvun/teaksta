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
import org.apache.tika.Tika;
import org.xml.sax.ContentHandler;
import org.apache.tika.sax.BodyContentHandler;
import org.apache.tika.metadata.Metadata;
import org.apache.tika.parser.Parser;
import org.apache.tika.parser.AutoDetectParser;
import org.apache.tika.parser.ParseContext;
import org.xml.sax.SAXException;
import org.apache.tika.exception.TikaException;

import org.jsoup.Connection;
import org.jsoup.Jsoup;
import org.jsoup.nodes.Node;
import org.jsoup.nodes.Document;

import java.io.InputStreamReader;
import java.util.Scanner;
//import java.nio.file.*;
import javax.servlet.http.HttpSession;
import java.io.Writer;
import java.io.BufferedWriter;
import java.io.OutputStreamWriter;
import java.io.FileWriter;
import java.io.BufferedReader;
import java.io.FileReader;

import werti.util.VerifyRecaptcha;

public class UploadDownloadFileServlet extends HttpServlet {
	private static final long serialVersionUID = 15;
  private ServletFileUpload uploader = null;

	@Override
	public void init() throws ServletException{
    DiskFileItemFactory fileFactory = new DiskFileItemFactory();
		File filesDir = (File) getServletContext().getAttribute("FILES_DIR_FILE");
    fileFactory.setRepository(filesDir);
    this.uploader = new ServletFileUpload(fileFactory);
	}

	public static boolean checkMetaData(File f, String getContentType) {
		 try {
			 InputStream is = new FileInputStream(f);
			 ContentHandler contenthandler = new BodyContentHandler();
			 Metadata metadata = new Metadata();
			 metadata.set(Metadata.RESOURCE_NAME_KEY, f.getName());
			 Parser parser = new AutoDetectParser();
			 try {
				 parser.parse(is, contenthandler, metadata, new ParseContext());
			 } catch (SAXException e) {
				 System.out.println("SAXException");
				 // Handle error
				 return false;
			 } catch (TikaException e) {
				 System.out.println("TikaException");
				 return false;
			 }
			 System.out.println("content_type="+metadata.get(Metadata.CONTENT_TYPE));


			 if (metadata.get(Metadata.CONTENT_TYPE).contains(getContentType)) {
				 return true;
			 } else {
				 return false;
			 }
		 } catch (IOException e) {
			 // Handle error
			 return false;
		 }
	 }


	protected void doPost(HttpServletRequest request, HttpServletResponse response) throws ServletException, IOException {
		if(!ServletFileUpload.isMultipartContent(request)){
			throw new ServletException("Content type is not multipart/form-data");
		}

		response.setContentType("text/html");
		response.setCharacterEncoding("UTF-8");

		PrintWriter out = response.getWriter();
		out.write("<html xmlns='http://www.w3.org/1999/xhtml' xml:lang='en' lang='sme'><head><meta charset='utf-8'></head><body>");
		try {
			String act = "";
			String en = "";
			String path = "";
			String captcha = "";
			boolean file_size_err = false;
			boolean file_type_err = false;
			boolean captcha_err = false;
			boolean verify = false;
			boolean file_uploaded = false;
			boolean save_file = false;
			boolean file_lang_err = false;
			int i = 0;
			int file_index = 0;
			List<FileItem> fileItemsList = uploader.parseRequest(request);
			Iterator<FileItem> fileItemsIterator = fileItemsList.iterator();
			while(fileItemsIterator.hasNext()){
				FileItem fileItem = fileItemsIterator.next();
				if (fileItem.isFormField()) {
			    String name = fileItem.getFieldName();
			    String value = fileItem.getString();
					System.out.println("name= " + name + "i= " + i + "value= " + value);
					if (name.equals("enhancement_upload")) {
						en = value;
					} else if (name.equals("activity_up")) {
							act = value;
						} else if (name.equals("g-recaptcha-response")){
							captcha = value;
							File secret_file = new File(getServletContext().getInitParameter("secret_key"));
							BufferedReader secret_reader = new BufferedReader(new FileReader(secret_file));
							String secret = secret_reader.readLine();
							verify = VerifyRecaptcha.verify(captcha, secret);
						} else if (name.equals("save_options")) {
							if (value.equals("save")) {
								save_file = true;
							}
						}
				} else {
					String name_f = fileItem.getFieldName();
			    String value_f = fileItem.getString();
					file_index = i;
				}
				i += 1;
			}
			FileItem file_item = fileItemsList.get(file_index);
			if (file_item.getString() != null && !file_item.getString().isEmpty()) {
				file_uploaded = true;
			}
			if (file_uploaded) {
				if (verify) {
					if (file_item.getSize() < 5*(1024*1024)) {
						InputStream file_input_stream = file_item.getInputStream();
						String rnd_name = RandomStringUtils.randomAlphanumeric(10);
						File file;
						if (save_file) {
							file = new File(getServletContext().getAttribute("FILES_PRM_DIR")+File.separator+rnd_name);
							path = file.getAbsolutePath();
						} else {
							file = new File(getServletContext().getAttribute("FILES_TMP_DIR")+File.separator+rnd_name);
							path = file.getAbsolutePath();
						}
						file_item.write(file);
						file.setExecutable(false);
						file.setReadable(true);
						file.setWritable(false);

						boolean textHtml = checkMetaData(file, "text/html");
						boolean textXhtml = checkMetaData(file, "application/xhtml+xml");

						if  ((textHtml) || (textXhtml)) {
							System.out.println("file is html");
							Document doc = Jsoup.parse(file,  "UTF-8");
							String textContent=doc.text();

							Writer writer = null;
							String filePath = "/tmp/inputLang.txt";
							try {
							    writer = new BufferedWriter(new OutputStreamWriter(
							          new FileOutputStream(filePath), "utf-8"));
							    writer.write(textContent);
							} catch (IOException ex) {
							    System.out.println("file not written");
							} finally {
							   try {writer.close();} catch (Exception ex) {/*ignore*/}
							}
							String check_lang_res = "";
							// pytextcat needs file -- filePath
							String[] text_check_lang = {"/bin/sh", "-c", " pytextcat proc <" + filePath };
							Process process = Runtime.getRuntime().exec(text_check_lang);
							BufferedReader stdInput = new BufferedReader(new InputStreamReader(process.getInputStream()));
							StringBuffer stdout = new StringBuffer();

								// read the output from the command
								String s = null;
								while ((s = stdInput.readLine()) != null) {
									stdout.append(s);
								}

								if (stdout.toString().equals("sme")) {
									System.out.println("file is sme");
								} else {
									//Files.deleteIfExists(Paths.get(path));
									//Files.deleteIfExists(Paths.get(path));
									if (file.exists()) {
										file.delete();
									}
									System.out.println("file not sme, deleted");
									path = "";
									file_lang_err = true;
								}
						} else {
							//Files.deleteIfExists(Paths.get(path));
							file.delete();
							path = "";
							file_type_err = true;
						}
					} else {
						path = "";
						file_size_err = true;
					}
				} else {
					path = "";
					captcha_err = true;
				}
			} else {
				path = "";
			}

			String host_name = "http://gtoahpa-01.uit.no/konteaksta"; //http://gtoahpa-01.uit.no , http://oahpa.no , http://127.0.0.1:8080

			if (!path.equals("")) {
				response.sendRedirect(host_name+"/WERTiServlet?activity="+act+"&client.enhancement="+en+"&url=file://"+path);
			} else {
				if (!file_uploaded) {
	        out.write("<br><br><br>" );
					out.write("<center><h1 style='color:#144ea6;'>Vajálduhttet sáddet fiilla!</h1></center>" );
					out.write("<center><a href="+host_name+">Ruovttoluotta</a></center>");
				}
				if (file_size_err) {
					out.write("<br><br><br>" );
					out.write("<center><h1 style='color:#144ea6;'>Fiila lea menddo stuoris! Lobálaš sturrodat: 5MB.</h1></center>" );
					out.write("<center><a href="+host_name+">Ruovttoluotta</a></center>");
				}
				if (file_type_err) {
					out.write( "<br><br><br>" );
					out.write( "<center><h1 style='color:#144ea6;'>Fiilla formáhta ii leat html! Lobálaš formáhta: html.</h1></center>" );
					out.write("<center><a href="+host_name+">Ruovttoluotta</a></center>");
				}
				if (captcha_err) {
					out.write( "<br><br><br>" );
					out.write( "<center><h1 style='color:#144ea6;'>Vajálduhttet captcha!</h1></center>" );
					out.write("<center><a href="+host_name+">Ruovttoluotta</a></center>");
				}
				if (file_lang_err) {
					out.write( "<br><br><br>" );
					out.write( "<center><h1 style='color:#144ea6;'>Fiila ii sisttisdoala davvisámegiela!</h1></center>" );
					out.write("<center><a href="+host_name+">Ruovttoluotta</a></center>");
				}
			}
		} catch (FileUploadException e) {
			out.write("Exception in uploading file. Cause: "+e.getCause());
		} catch (Exception e) {
			out.write("Exception in uploading file. Cause: "+e.getCause());
		}
		out.write("</body></html>");
	}
}
