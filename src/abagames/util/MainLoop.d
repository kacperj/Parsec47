/*
 * $Id: MainLoop.d,v 1.3 2004/01/01 11:26:43 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.util.sdl.MainLoop;

private:
import abagames.util.Logger;
import abagames.util.Rand;
import abagames.p47.P47PrefManager;
import abagames.p47.P47GameManager;
import abagames.util.sdl.Pad;

private extern(C):
int  window_poll_events();
int  window_get_resize_w();
int  window_get_resize_h();
uint window_get_ticks();
void window_delay(uint ms);
int  screen_init_sdl(int lowres, int windowMode, int fullscreenDesktop, float luminous);
void screen_resized(int width, int height);
void screen_clear();
int  screen_flip();
void screen_close_sdl();

/**
 * SDL main loop.
 */
public class MainLoop
{
public:
  const int INTERVAL_BASE = 16;
  int interval = INTERVAL_BASE;
  int accframe = 0;
  int maxSkipFrame = 5;
  // Screen configuration, set from command-line options before loop() runs.
  static bool lowres = false;
  static bool windowMode = false;
  static bool fullscreenDesktop = false;
  static float luminous = 0;

private:
  P47GameManager gameManager;

  public this(P47GameManager gameManager)
  {
    gameManager.setMainLoop(this);
    this.gameManager = gameManager;
  }

  // Initialize and load preference.
  private void initFirst()
  {
    prefs_load();
    gameManager.init();
  }

  // Quit and save preference.
  private void quitLast()
  {
    gameManager.close();
    prefs_save();
    screen_close_sdl();
  }

  private bool done;

  public void breakLoop()
  {
    done = true;
  }

  public void loop()
  {
    done = false;
    long prvTickCount = 0;
    int i;
    long nowTick;
    int frame;

    if (screen_init_sdl(lowres ? 1 : 0, windowMode ? 1 : 0,
        fullscreenDesktop ? 1 : 0, luminous) < 0)
      throw new Exception("Unable to create window");
    initFirst();
    gameManager.start();

    while (!done)
    {
      int evMask = window_poll_events();
      if (evMask & 1)
        breakLoop();
      if (evMask & 2)
        screen_resized(window_get_resize_w(), window_get_resize_h());
      nowTick = window_get_ticks();
      frame = cast(int)(nowTick - prvTickCount) / interval;
      if (frame <= 0)
      {
        frame = 1;
        window_delay(cast(uint)(prvTickCount + interval - nowTick));
        if (accframe)
        {
          prvTickCount = window_get_ticks();
        }
        else
        {
          prvTickCount += interval;
        }
      }
      else if (frame > maxSkipFrame)
      {
        frame = maxSkipFrame;
        prvTickCount = nowTick;
      }
      else
      {
        prvTickCount += frame * interval;
      }
      for (i = 0; i < frame; i++)
      {
        gameManager.move();
      }
      screen_clear();
      gameManager.draw();
      if (screen_flip() < 0)
        throw new Exception("OpenGL error");
    }
    quitLast();
  }
}
