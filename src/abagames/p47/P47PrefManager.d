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
  int* prefs_hi_score_ptr();
  int* prefs_reached_parsec_ptr();
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
  static const int REACHED_PARSEC_SLOT_NUM = 10;

  @property ref int[REACHED_PARSEC_SLOT_NUM][DIFFICULTY_NUM][MODE_NUM] hiScore()
  {
    return *cast(int[REACHED_PARSEC_SLOT_NUM][DIFFICULTY_NUM][MODE_NUM]*) prefs_hi_score_ptr();
  }

  @property ref int[DIFFICULTY_NUM][MODE_NUM] reachedParsec()
  {
    return *cast(int[DIFFICULTY_NUM][MODE_NUM]*) prefs_reached_parsec_ptr();
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
