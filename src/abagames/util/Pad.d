/*
 * $Id: Pad.d,v 1.3 2004/01/01 11:26:43 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.util.sdl.Pad;

public extern(C):
int  pad_open_joystick();
int  pad_get_pad_state();
int  pad_get_button_state();
void pad_set_button_reversed(int v);
int  pad_get_button_reversed();
int  pad_is_key_pressed(int sk);

/**
 * Joystick and keyboard input (delegated to Rust/SDL2).
 */
public class Pad
{
public:
  static const int PAD_UP      = 1;
  static const int PAD_DOWN    = 2;
  static const int PAD_LEFT    = 4;
  static const int PAD_RIGHT   = 8;
  static const int PAD_BUTTON1 = 16;
  static const int PAD_BUTTON2 = 32;

public:

  @property bool buttonReversed()       { return pad_get_button_reversed() != 0; }

  // SDL2 SDLK values: SDLK_p=112, SDLK_ESCAPE=27 (same as SDL1 for ASCII keys)
  bool isPausePressed()  { return pad_is_key_pressed(112) != 0; }

  bool isAnyDirectionPressed(){ return pad_get_pad_state()                    != 0;  }
  bool isButton1()            { return (pad_get_button_state() & PAD_BUTTON1) != 0; }
  bool isButton2()            { return (pad_get_button_state() & PAD_BUTTON2) != 0; }
}
