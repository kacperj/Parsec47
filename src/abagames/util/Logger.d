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
public class Logger
{

  public static void info(string msg)
  {
    stderr.writeln("Info: " ~ msg);
  }

  public static void error(string msg)
  {
    stderr.writeln("Error: " ~ msg);
  }
}
