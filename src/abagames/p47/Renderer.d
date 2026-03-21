module abagames.p47.Renderer;

struct Color
{
  float r;
  float g;
  float b;
  float a;
}

extern(C)
{
  void renderer_set_brightness(float b);
  float renderer_get_brightness();
  void renderer_set_color(float r, float g, float b, float a);
  void renderer_draw_box_solid(float x, float y, float width, float height);
  void renderer_draw_box_line(float x, float y, float width, float height);
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
}
