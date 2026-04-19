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
  void renderer_set_color_params(float r, float g, float b, float a);
  void draw_line_retro_with_z(float x1, float y1, float x2, float y2, float z,
                                      float retro, float retroSize,
                                      Color color);
  void draw_box_retro(float center_x, float center_y, float width, float height, float deg,
                               Color color,
                               float retro, float retroSize);
}

public class Renderer
{
  static @property float brightness() { return renderer_get_brightness(); }
  static @property void brightness(float b) { renderer_set_brightness(b); }

  public static void setColor(Color color)
  {
    renderer_set_color_params(color.r, color.g, color.b, color.a);
  }

  public static void drawBoxRetro(float center_x, float center_y, float width, float height, float deg, Color color, RetroParam param)
  {
    draw_box_retro(center_x, center_y, width, height, deg, color, param.retro, param.retroSize);
  }

  public static void drawLineRetro(float x1, float y1, float x2, float y2, float z, Color color, RetroParam param)
  {
    draw_line_retro_with_z(x1, y1, x2, y2, z, param.retro, param.retroSize, color);
  }
}
