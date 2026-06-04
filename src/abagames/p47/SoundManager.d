/*
 * $Id: SoundManager.d,v 1.3 2004/01/01 11:26:42 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.SoundManager;

public extern (C)
{
  int sound_manager_init();
  void sound_manager_close();
  void sound_manager_set_no_sound(int v);
  void sound_manager_set_in_game(int v);
  void sound_manager_play_bgm(int n);
  void sound_manager_play_se(int n);
  void sound_manager_fade_music();
}

/**
 * Manage BGMs/SEs.
 */
public class SoundManager
{
public static:
  enum
  {
    SHOT,
    ROLL_CHARGE,
    ROLL_RELEASE,
    SHIP_DESTROYED,
    GET_BONUS,
    EXTEND,
    ENEMY_DESTROYED,
    LARGE_ENEMY_DESTROYED,
    BOSS_DESTROYED,
    LOCK,
    LASER,
  }
  const int BGM_NUM = 4;

  static @property void isInGame(bool v) { sound_manager_set_in_game(v ? 1 : 0); }

  public static void fadeMusic() { sound_manager_fade_music(); }
  public static void playSe(int n) { sound_manager_play_se(n); }
}
