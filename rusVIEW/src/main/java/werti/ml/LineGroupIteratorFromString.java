/* Copyright (C) 2002 Univ. of Massachusetts Amherst, Computer Science Dept.
   This file is part of "MALLET" (MAchine Learning for LanguagE Toolkit).
   http://www.cs.umass.edu/~mccallum/mallet
   This software is provided under the terms of the Common Public License,
   version 1.0, as published by http://www.opensource.org.  For further
   information, see the file `LICENSE' included with this distribution. */

package werti.ml;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.StringReader;
import java.util.Iterator;
import java.util.regex.Pattern;

import cc.mallet.types.Instance;

/**
 * Iterate over groups of lines of text, separated by lines that match a regular
 * expression.
 * 
 * This class is a very slightly modified version of LineGroupIterator that
 * accepts string input in place of file input. It could potentially extend
 * LineGroupIterator instead of copying it, but not without modifying
 * LineGroupIterator, so I decided it was preferable to work with a standard
 * version of mallet rather than to reduce the amount of duplicated code.
 * 
 * @author MALLET
 * @author Adriane Boyd
 */

public class LineGroupIteratorFromString implements Iterator<Instance> {
	BufferedReader reader;
	Pattern lineBoundaryRegex;
	boolean skipBoundary;
	String nextLineGroup;
	String nextBoundary;
	String nextNextBoundary;
	int groupIndex = 0;
	boolean putBoundaryInSource = true;

	public LineGroupIteratorFromString(String inputString,
			Pattern lineBoundaryRegex, boolean skipBoundary) {
		this.reader = new BufferedReader(new StringReader(inputString));
		this.lineBoundaryRegex = lineBoundaryRegex;
		this.skipBoundary = skipBoundary;
		setNextLineGroup();
	}

	public String peekLineGroup() {
		return nextLineGroup;
	}

	private void setNextLineGroup() {
		StringBuffer sb = new StringBuffer();
		String line;
		if (!skipBoundary && nextBoundary != null)
			sb.append(nextBoundary + '\n');
		while (true) {
			try {
				line = reader.readLine();
			} catch (IOException e) {
				throw new RuntimeException(e);
			}
			if (line == null) {
				break;
			} else if (lineBoundaryRegex.matcher(line).matches()) {
				if (sb.length() > 0) {
					this.nextBoundary = this.nextNextBoundary;
					this.nextNextBoundary = line;
					break;
				} else { // The first line of the file.
					if (!skipBoundary)
						sb.append(line + '\n');
					this.nextNextBoundary = line;
				}
			} else {
				sb.append(line);
				sb.append('\n');
			}
		}
		if (sb.length() == 0)
			this.nextLineGroup = null;
		else
			this.nextLineGroup = sb.toString();
	}

	@Override
	public Instance next() {
		assert (nextLineGroup != null);
		Instance carrier = new Instance(nextLineGroup, null, "linegroup"
				+ groupIndex++, putBoundaryInSource ? nextBoundary : null);
		setNextLineGroup();
		return carrier;
	}

	@Override
	public boolean hasNext() {
		return nextLineGroup != null;
	}

	@Override
	public void remove() {
		throw new IllegalStateException(
				"This Iterator<Instance> does not support remove().");
	}

}