/*
 * $Id: Pad.d,v 1.3 2004/01/01 11:26:43 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.util.sdl.Pad;

private:
import SDL;
import abagames.util.sdl.SDLInitFailedException;

private extern(C):
int  pad_open_joystick();
void pad_handle_event(void* event);
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
private:
  static const int PAD_UP      = 1;
  static const int PAD_DOWN    = 2;
  static const int PAD_LEFT    = 4;
  static const int PAD_RIGHT   = 8;
  static const int PAD_BUTTON1 = 16;
  static const int PAD_BUTTON2 = 32;

public:

  @property bool buttonReversed()       { return pad_get_button_reversed() != 0; }
  @property void buttonReversed(bool v) { pad_set_button_reversed(v ? 1 : 0); }

  // SDL1 SDLKey values: SDLK_p=112, SDLK_ESCAPE=27
  bool isPausePressed()  { return pad_is_key_pressed(112) != 0; }
  bool isEscapePressed() { return pad_is_key_pressed(27) != 0; }

  public void openJoystick()
  {
    if (pad_open_joystick() < 0)
    {
      throw new SDLInitFailedException("Unable to init SDL joystick");
    }
  }

  public void handleEvent(SDL_Event* event)
  {
    pad_handle_event(cast(void*) event);
  }

  bool isPadUp()              { return (pad_get_pad_state()    & PAD_UP)      != 0; }
  bool isPadDown()            { return (pad_get_pad_state()    & PAD_DOWN)    != 0; }
  bool isPadLeft()            { return (pad_get_pad_state()    & PAD_LEFT)    != 0; }
  bool isPadRight()           { return (pad_get_pad_state()    & PAD_RIGHT)   != 0; }
  bool isAnyDirectionPressed(){ return pad_get_pad_state()                    != 0;  }
  bool isButton1()            { return (pad_get_button_state() & PAD_BUTTON1) != 0; }
  bool isButton2()            { return (pad_get_button_state() & PAD_BUTTON2) != 0; }
}
