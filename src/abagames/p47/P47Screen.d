/*
 * $Id: P47Screen.d,v 1.5 2004/01/01 11:26:42 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.P47Screen;

private:
import std.string;
import std.math;
import SDL;
import opengl;
import abagames.util.Logger;
import abagames.util.sdl.SDLInitFailedException;
import abagames.p47.LuminousScreen;
import abagames.p47.Renderer;

extern(C) void draw_line_retro_with_z(float x1, float y1, float x2, float y2, float z,
                                      float retro, float retroSize,
                                      float r, float g, float b, float a);

/**
 * SDL screen handler and OpenGL setup for PARSEC47.
 */
public class P47Screen
{
public:
  static const string CAPTION = "PARSEC47";
  static float luminous = 0;
  static int width = 640;
  static int height = 480;
  static bool lowres = false;
  static bool windowMode = false;
  static float nearPlane = 0.1;
  static float farPlane = 1000;
private:
  LuminousScreen luminousScreen;

  public void initSDL()
  {
    if (lowres)
    {
      width /= 2;
      height /= 2;
    }
    if (SDL_Init(SDL_INIT_VIDEO) < 0)
    {
      throw new SDLInitFailedException(
        "Unable to initialize SDL: " ~ std.string.fromStringz(SDL_GetError()).idup);
    }
    Uint32 videoFlags;
    if (windowMode)
    {
      videoFlags = SDL_OPENGL | SDL_RESIZABLE;
    }
    else
    {
      videoFlags = SDL_OPENGL | SDL_FULLSCREEN;
    }
    if (SDL_SetVideoMode(width, height, 0, videoFlags) == null)
    {
      throw new SDLInitFailedException(
        "Unable to create SDL screen: " ~ std.string.fromStringz(SDL_GetError()).idup);
    }
    glClearColor(0.0f, 0.0f, 0.0f, 0.0f);
    resized(width, height);
    SDL_ShowCursor(SDL_DISABLE);
    init();
  }

  private void init()
  {
    SDL_WM_SetCaption(cast(char*) std.string.toStringz(CAPTION), null);
    glLineWidth(1);
    glEnable(GL_LINE_SMOOTH);
    glBlendFunc(GL_SRC_ALPHA, GL_ONE);
    glEnable(GL_BLEND);
    glDisable(GL_LIGHTING);
    glDisable(GL_CULL_FACE);
    glDisable(GL_DEPTH_TEST);
    glDisable(GL_TEXTURE_2D);
    glDisable(GL_COLOR_MATERIAL);
    if (luminous > 0)
    {
      luminousScreen = new LuminousScreen;
      luminousScreen.init(luminous, width, height);
    }
    else
    {
      luminousScreen = null;
    }
  }

  private void close()
  {
    if (luminousScreen)
      luminousScreen.close();
  }

  private void screenResized()
  {
    glViewport(0, 0, width, height);
    glMatrixMode(GL_PROJECTION);
    glLoadIdentity();
    glFrustum(-nearPlane,
      nearPlane,
      -nearPlane * cast(GLfloat) height / cast(GLfloat) width,
      nearPlane * cast(GLfloat) height / cast(GLfloat) width,
      0.1f, farPlane);
    glMatrixMode(GL_MODELVIEW);
  }

  public void resized(int width, int height)
  {
    this.width = width;
    this.height = height;
    if (luminousScreen)
      luminousScreen.resized(width, height);
    screenResized();
  }

  public void closeSDL()
  {
    close();
    SDL_ShowCursor(SDL_ENABLE);
  }

  public void flip()
  {
    handleError();
    SDL_GL_SwapBuffers();
  }

  public void clear()
  {
    glClear(GL_COLOR_BUFFER_BIT);
  }

  public void handleError()
  {
    GLenum error = glGetError();
    if (error == GL_NO_ERROR)
      return;
    closeSDL();
    throw new Exception("OpenGL error");
  }

  public void startRenderToTexture()
  {
    if (luminousScreen)
      luminousScreen.startRenderToTexture();
  }

  public void endRenderToTexture()
  {
    if (luminousScreen)
      luminousScreen.endRenderToTexture();
  }

  public void drawLuminous()
  {
    if (luminousScreen)
      luminousScreen.draw();
  }

  public void viewOrthoFixed()
  {
    glMatrixMode(GL_PROJECTION);
    glPushMatrix();
    glLoadIdentity();
    glOrtho(0, 640, 480, 0, -1, 1);
    glMatrixMode(GL_MODELVIEW);
    glPushMatrix();
    glLoadIdentity();
  }

  public void viewPerspective()
  {
    glMatrixMode(GL_PROJECTION);
    glPopMatrix();
    glMatrixMode(GL_MODELVIEW);
    glPopMatrix();
  }

  // Draw the retro style lines.
  private static float retro, retroSize;
  private static Color retroColor;

  public static void setRetroParam(float r, float sz)
  {
    retro = r;
    retroSize = sz;
  }

  public static void setRetroColor(Color color)
  {
    retroColor = color;
  }

  public static void drawBoxRetro(float x, float y, float width, float height, float deg)
  {
    float w1, h1, w2, h2;
    w1 = width * cos(deg) - height * sin(deg);
    h1 = width * sin(deg) + height * cos(deg);
    w2 = -width * cos(deg) - height * sin(deg);
    h2 = -width * sin(deg) + height * cos(deg);
    drawLineRetro(x + w2, y - h2, x + w1, y - h1);
    drawLineRetro(x + w1, y - h1, x - w2, y + h2);
    drawLineRetro(x - w2, y + h2, x - w1, y + h1);
    drawLineRetro(x - w1, y + h1, x + w2, y - h2);
  }

  public static void drawLineRetro(float x1, float y1, float x2, float y2)
  {
    drawLineRetroWithZ(x1, y1, x2, y2, 0);
  }

  public static void drawLineRetroWithZ(float x1, float y1, float x2, float y2, float z)
  {
    draw_line_retro_with_z(x1, y1, x2, y2, z, retro, retroSize,
                           retroColor.r, retroColor.g, retroColor.b, retroColor.a);
  }

}
