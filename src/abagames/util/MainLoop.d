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
import abagames.p47.P47Screen;
import abagames.util.sdl.Pad;

private extern(C):
int  window_poll_events();
int  window_get_resize_w();
int  window_get_resize_h();
uint window_get_ticks();
void window_delay(uint ms);

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

private:
  P47Screen screen;
  Pad input;
  P47GameManager gameManager;
  P47PrefManager prefManager;

  public this(P47Screen screen, Pad input,
    P47GameManager gameManager, P47PrefManager prefManager)
  {
    this.screen = screen;
    this.input = input;
    gameManager.setMainLoop(this);
    gameManager.setUIs(screen, input);
    gameManager.setPrefManager(prefManager);
    this.gameManager = gameManager;
    this.prefManager = prefManager;
  }

  // Initialize and load preference.
  private void initFirst()
  {
    prefManager.load();
    gameManager.init();
  }

  // Quit and save preference.
  private void quitLast()
  {
    gameManager.close();
    prefManager.save();
    screen.closeSDL();
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

    screen.initSDL();
    initFirst();
    gameManager.start();

    while (!done)
    {
      int evMask = window_poll_events();
      if (evMask & 1)
        breakLoop();
      if (evMask & 2)
        screen.resized(window_get_resize_w(), window_get_resize_h());
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
      screen.clear();
      gameManager.draw();
      screen.flip();
    }
    quitLast();
  }
}
