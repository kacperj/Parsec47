/*
 * $Id: GameManager.d,v 1.2 2004/01/01 11:26:43 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.util.sdl.GameManager;

private:
import abagames.p47.P47PrefManager;
import abagames.util.sdl.MainLoop;
import abagames.util.sdl.Screen3D;
import abagames.util.sdl.Pad;

/**
 * Manage the lifecycle of the game.
 */
public class GameManager
{
public:
  int status;

protected:
  MainLoop mainLoop;
  Screen3D abstScreen;
  Pad input;
  P47PrefManager prefManager;

private:

  public void setMainLoop(MainLoop mainLoop)
  {
    this.mainLoop = mainLoop;
  }

  public void setUIs(Screen3D screen, Pad input)
  {
    abstScreen = screen;
    this.input = input;
  }

  public void setPrefManager(P47PrefManager prefManager)
  {
    this.prefManager = prefManager;
  }

  public abstract void init();
  public abstract void start();
  public abstract void close();
  public abstract void move();
  public abstract void draw();
}
