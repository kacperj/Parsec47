module abagames.p47.LetterRender;

private extern(C):
void letter_render_create_display_lists();
void letter_render_draw_string(const(ubyte)* ptr, int len, float lx, float y, float s, int d);
void letter_render_draw_num(int num, float lx, float y, float s, int d);

public class LetterRender
{
  public enum
  {
    TO_RIGHT,
    TO_DOWN,
    TO_LEFT,
    TO_UP,
  }

  public static void createDisplayLists()
  {
    letter_render_create_display_lists();
  }

  public static void drawString(string str, float lx, float y, float s, int d)
  {
    letter_render_draw_string(cast(const(ubyte)*) str.ptr, cast(int) str.length, lx, y, s, d);
  }
}
