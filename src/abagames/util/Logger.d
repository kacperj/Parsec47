/*
 * $Id: Logger.d,v 1.2 2004/01/01 11:26:43 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.util.Logger;

private:
import std.stdio;

/**
 * Logger(error/info).
 */
import std.conv: to;

public class Logger {

  public static void info(string msg) {
    stderr.writeln("Info: " ~ msg);
  }

  public static void info(int n) {
    if (n >= 0)
      stderr.writeln("Info: " ~ to!string(n));
    else
      stderr.writeln("Info: -" ~ to!string(-n));
  }

  public static void error(string msg) {
    stderr.writeln("Error: " ~ msg);
  }

  public static void error(Exception e) {
    stderr.writeln("Error: " ~ e.toString());
  }

  public static void error(Error e) {
    stderr.writeln("Error: " ~ e.toString());
  }
}
