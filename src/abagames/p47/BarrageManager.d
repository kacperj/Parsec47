/*
 * $Id: BarrageManager.d,v 1.2 2004/01/01 11:26:41 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.BarrageManager;

private:
import std.file : dirEntries, SpanMode;
import std.path : baseName, extension;
import std.string;
import bulletml;
import abagames.util.Logger;

/**
 * Barrage manager(BulletMLs' loader).
 */
public class BarrageManager
{
public:
  enum
  {
    MORPH,
    SMALL,
    SMALLMOVE,
    SMALLSIDEMOVE,
    MIDDLE,
    MIDDLESUB,
    MIDDLEMOVE,
    MIDDLEBACKMOVE,
    LARGE,
    LARGEMOVE,
    MORPH_LOCK,
    SMALL_LOCK,
    MIDDLESUB_LOCK,
  }
  const int BARRAGE_TYPE = 13;
  static const int BARRAGE_MAX = 64;
  BulletMLParserTinyXML*[BARRAGE_MAX][BARRAGE_TYPE] parser;
  int[BARRAGE_TYPE] parserNum;
private:
  const string[BARRAGE_TYPE] dirName =
    [
      "morph", "small", "smallmove", "smallsidemove",
      "middle", "middlesub", "middlemove", "middlebackmove",
      "large", "largemove",
      "morph_lock", "small_lock", "middlesub_lock"
    ];

  public void loadBulletMLs()
  {
    for (int i = 0; i < BARRAGE_TYPE; i++)
    {
      int j = 0;
      string dir = "assets/bulletdata/" ~ dirName[i];
      foreach (entry; dirEntries(dir, SpanMode.shallow))
      {
        string fileName = baseName(entry.name);
        if (extension(fileName) != ".xml")
          continue;
        Logger.info("Load BulletML: " ~ dir ~ "/" ~ fileName);
        parser[i][j] =
          BulletMLParserTinyXML_new(cast(char*) std.string.toStringz(dir ~ "/" ~ fileName));
        BulletMLParserTinyXML_parse(parser[i][j]);
        j++;
      }
      parserNum[i] = j;
    }
  }

  public void unloadBulletMLs()
  {
    for (int i = 0; i < BARRAGE_TYPE; i++)
    {
      for (int j = 0; j < parserNum[i]; j++)
      {
        BulletMLParserTinyXML_delete(parser[i][j]);
      }
    }
  }
}
