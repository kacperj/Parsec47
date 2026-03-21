module abagames.util.sdl.Sound;

private import std.string;
private import abagames.util.sdl.SDLInitFailedException;

private extern (C)
{
  int sound_init();
  void sound_close();
  void sound_set_no_sound(int v);
  int sound_get_no_sound();
  void sound_set_fade_out_speed(int speed);
  int sound_alloc_slot();
  int sound_load_music(int slot, const(char)* path);
  int sound_load_chunk(int slot, const(char)* path, int channel);
  void sound_free_slot(int slot);
  void sound_play_music(int slot);
  void sound_play_chunk(int slot);
  void sound_halt_chunk(int slot);
  void sound_fade_music();
  void sound_stop_music();
}

public class Sound
{
public:
  static @property bool noSound() { return sound_get_no_sound() != 0; }
  static @property void noSound(bool v) { sound_set_no_sound(v ? 1 : 0); }

  static int fadeOutSpeed = 1280;
  static string soundsDir = "assets/sounds/";
  static string chunksDir = "assets/sounds/";

  public static void init()
  {
    if (noSound)
      return;
    sound_set_fade_out_speed(fadeOutSpeed);
    if (sound_init() < 0)
    {
      throw new SDLInitFailedException("Failed to initialize SDL2 audio");
    }
  }

  public static void close()
  {
    if (noSound)
      return;
    sound_close();
  }

  public static void fadeMusic()
  {
    if (noSound)
      return;
    sound_fade_music();
  }

  public static void stopMusic()
  {
    if (noSound)
      return;
    sound_stop_music();
  }

private:
  int slot = -1;

public:
  this()
  {
    slot = sound_alloc_slot();
  }

  public void loadSound(string name)
  {
    if (noSound)
      return;
    string fileName = soundsDir ~ name;
    if (sound_load_music(slot, std.string.toStringz(fileName)) < 0)
    {
      noSound = true;
      throw new SDLInitFailedException("Couldn't load: " ~ fileName);
    }
  }

  public void loadChunk(string name, int ch)
  {
    if (noSound)
      return;
    string fileName = chunksDir ~ name;
    if (sound_load_chunk(slot, std.string.toStringz(fileName), ch) < 0)
    {
      noSound = true;
      throw new SDLInitFailedException("Couldn't load: " ~ fileName);
    }
  }

  public void free()
  {
    if (slot >= 0)
      sound_free_slot(slot);
  }

  public void playMusic()
  {
    if (noSound)
      return;
    sound_play_music(slot);
  }

  public void playChunk()
  {
    if (noSound)
      return;
    sound_play_chunk(slot);
  }

  public void haltChunk()
  {
    if (noSound)
      return;
    sound_halt_chunk(slot);
  }
}
