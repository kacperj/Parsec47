module abagames.p47.Renderer;

struct Color
{
  float r;
  float g;
  float b;
  float a;
}

public struct RetroParam
{
  float retro;
  float retroSize;
}

extern(C)
{
  void renderer_set_brightness(float b);
  float renderer_get_brightness();
  void renderer_set_color(float r, float g, float b, float a);
  void renderer_draw_box_solid(float x, float y, float width, float height);
  void renderer_draw_box_line(float x, float y, float width, float height);
  void draw_line_retro_with_z(float x1, float y1, float x2, float y2, float z,
                                      float retro, float retroSize,
                                      Color color);
  void draw_box_retro(float x, float y, float width, float height, float deg,
                               Color color,
                               float retro, float retroSize);
}

public class Renderer
{
  static @property float brightness() { return renderer_get_brightness(); }
  static @property void brightness(float b) { renderer_set_brightness(b); }

  public static void setColor(Color color)
  {
    renderer_set_color(color.r, color.g, color.b, color.a);
  }

  public static void drawBoxSolid(float x, float y, float width, float height)
  {
    renderer_draw_box_solid(x, y, width, height);
  }

  public static void drawBoxLine(float x, float y, float width, float height)
  {
    renderer_draw_box_line(x, y, width, height);
  }

  public static void drawBoxRetro(float x, float y, float width, float height, float deg, Color color, RetroParam param)
  {
    draw_box_retro(x, y, width, height, deg, color, param.retro, param.retroSize);
  }

  public static void drawLineRetro(float x1, float y1, float x2, float y2, float z, Color color, RetroParam param)
  {
    draw_line_retro_with_z(x1, y1, x2, y2, z, param.retro, param.retroSize, color);
  }
}
