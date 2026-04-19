/*
 * $Id: P47PrefManager.d,v 1.2 2004/01/01 11:26:42 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.P47PrefManager;

extern (C)
{
  void prefs_load();
  void prefs_save();
  int prefs_get_hi_score(int mode, int difficulty, int slot);
  void prefs_set_hi_score(int mode, int difficulty, int slot, int val);
  int prefs_get_reached_parsec(int mode, int difficulty);
  void prefs_set_reached_parsec(int mode, int difficulty, int val);
  int prefs_get_selected_difficulty();
  void prefs_set_selected_difficulty(int val);
  int prefs_get_selected_parsec_slot();
  void prefs_set_selected_parsec_slot(int val);
  int prefs_get_selected_mode();
  void prefs_set_selected_mode(int val);
}

/**
 * Save/Load the high score.
 * Data is owned by the Rust `prefs` crate (persisted as p47.json).
 * This class is a pure proxy — it holds no state of its own.
 */
public class P47PrefManager
{
public:
  static const int MODE_NUM = 2;
  static const int DIFFICULTY_NUM = 4;

  int getHiScore(int mode, int difficulty, int slot)
  {
    return prefs_get_hi_score(mode, difficulty, slot);
  }

  void setHiScore(int mode, int difficulty, int slot, int val)
  {
    prefs_set_hi_score(mode, difficulty, slot, val);
  }

  int getReachedParsec(int mode, int difficulty)
  {
    return prefs_get_reached_parsec(mode, difficulty);
  }

  void setReachedParsec(int mode, int difficulty, int val)
  {
    prefs_set_reached_parsec(mode, difficulty, val);
  }

  @property int selectedDifficulty() { return prefs_get_selected_difficulty(); }
  @property void selectedDifficulty(int v) { prefs_set_selected_difficulty(v); }

  @property int selectedParsecSlot() { return prefs_get_selected_parsec_slot(); }
  @property void selectedParsecSlot(int v) { prefs_set_selected_parsec_slot(v); }

  @property int selectedMode() { return prefs_get_selected_mode(); }
  @property void selectedMode(int v) { prefs_set_selected_mode(v); }

  void load() { prefs_load(); }
  void save() { prefs_save(); }
}
