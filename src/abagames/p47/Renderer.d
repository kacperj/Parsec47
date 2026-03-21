module abagames.p47.Renderer;

import std.math;
import opengl;

public class Renderer
{
  static float brightness = 1;

  public static void setColor(float r, float g, float b, float a)
  {
    glColor4f(r * brightness, g * brightness, b * brightness, a);
  }

  public static void drawBoxSolid(float x, float y, float width, float height)
  {
    glBegin(GL_TRIANGLE_FAN);
    glVertex3f(x, y, 0);
    glVertex3f(x + width, y, 0);
    glVertex3f(x + width, y + height, 0);
    glVertex3f(x, y + height, 0);
    glEnd();
  }

  public static void drawBoxLine(float x, float y, float width, float height)
  {
    glBegin(GL_LINE_LOOP);
    glVertex3f(x, y, 0);
    glVertex3f(x + width, y, 0);
    glVertex3f(x + width, y + height, 0);
    glVertex3f(x, y + height, 0);
    glEnd();
  }
}
