/*
 * $Id: Title.d,v 1.2 2004/01/01 11:26:43 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.Title;

public extern (C)
{
  void renderer_title_texture_init();
  void renderer_title_texture_delete();

  void title_start(int difficulty, int parsecSlot, int mode);
  void title_move();
  int  title_should_change_stage();
  int  title_get_cur_x();
  int  title_get_cur_y();
  int  title_get_mode();
  void title_change_mode();
  void title_draw();
}

public struct StageSelection
{
  int difficulty;
  int parsecSlot;
  int mode;
}

/**
 * Title.
 */
public class Title
{
  public StageSelection getStatus()
  {
    StageSelection s;
    s.difficulty = title_get_cur_y();
    s.parsecSlot = title_get_cur_x();
    s.mode       = title_get_mode();
    return s;
  }
}
