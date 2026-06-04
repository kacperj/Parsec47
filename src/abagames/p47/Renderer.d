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
  void renderer_set_color_params(float r, float g, float b, float a);
  void draw_line_retro_with_z(float x1, float y1, float x2, float y2, float z,
                                      float retro, float retroSize,
                                      Color color);
}

public class Renderer
{
  public static void setColor(Color color)
  {
    renderer_set_color_params(color.r, color.g, color.b, color.a);
  }

  public static void drawLineRetro(float x1, float y1, float x2, float y2, float z, Color color, RetroParam param)
  {
    draw_line_retro_with_z(x1, y1, x2, y2, z, param.retro, param.retroSize, color);
  }
}
