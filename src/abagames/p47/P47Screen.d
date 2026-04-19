/*
 * $Id: P47Screen.d,v 1.5 2004/01/01 11:26:42 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.P47Screen;

private:
import std.string;
import std.math;
import opengl;
import abagames.util.Logger;
import abagames.p47.Renderer;

private extern(C):
int  window_init(int width, int height, int fullscreen, const(char)* title);
void window_close();
void window_gl_swap();
void window_show_cursor(int show);
void luminous_screen_init(float luminous, int width, int height);
void luminous_screen_close();
void luminous_screen_resized(int width, int height);
void luminous_screen_start_render_to_texture();
void luminous_screen_end_render_to_texture();
void luminous_screen_draw();

/**
 * SDL screen handler and OpenGL setup for PARSEC47.
 */
public class P47Screen
{

private:
  static float nearPlane = 0.1;
  static float farPlane = 1000;
  static const string CAPTION = "PARSEC47";
  static int width = 640;
  static int height = 480;

public:
  static bool lowres = false;
  static bool windowMode = false;
  static bool fullscreenDesktop = false;
  static float luminous = 0;
  bool hasLuminous;

  public void initSDL()
  {
    if (lowres)
    {
      width /= 2;
      height /= 2;
    }
    int fullscreen = windowMode ? 0 : (fullscreenDesktop ? 2 : 1);
    if (window_init(width, height, fullscreen, std.string.toStringz(CAPTION)) < 0)
    {
      throw new Exception("Unable to create window");
    }
    glClearColor(0.0f, 0.0f, 0.0f, 0.0f);
    resized(width, height);
    window_show_cursor(0);
    init();
  }

  private void init()
  {
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
      luminous_screen_init(luminous, width, height);
      hasLuminous = true;
    }
    else
    {
      hasLuminous = false;
    }
  }

  private void close()
  {
    if (hasLuminous)
      luminous_screen_close();
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
    if (hasLuminous)
      luminous_screen_resized(width, height);
    screenResized();
  }

  public void closeSDL()
  {
    close();
    window_show_cursor(1);
    window_close();
  }

  public void flip()
  {
    handleError();
    window_gl_swap();
  }

  public void clear()
  {
    glClear(GL_COLOR_BUFFER_BIT);
  }

  public void startRenderToTexture()
  {
    if (hasLuminous)
      luminous_screen_start_render_to_texture();
  }

  public void endRenderToTexture()
  {
    if (hasLuminous)
      luminous_screen_end_render_to_texture();
  }

  public void drawLuminous()
  {
    if (hasLuminous)
      luminous_screen_draw();
  }

  public void handleError()
  {
    GLenum error = glGetError();
    if (error == GL_NO_ERROR)
      return;
    closeSDL();
    throw new Exception("OpenGL error");
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
}
