module abagames.util.sdl.Sound;

private import std.string;
private import abagames.util.sdl.SDLInitFailedException;

private extern (C)
{
  int sound_alloc_slot();
  int sound_load_music(int slot, const(char)* path);
  int sound_load_chunk(int slot, const(char)* path, int channel);
  void sound_free_slot(int slot);
  void sound_play_music(int slot);
  void sound_play_chunk(int slot);
}

public class Sound
{
public:
  static string soundsDir = "assets/sounds/";
  static string chunksDir = "assets/sounds/";

private:
  int slot = -1;

public:
  this()
  {
    slot = sound_alloc_slot();
  }

  public void loadSound(string name)
  {
    string fileName = soundsDir ~ name;
    if (sound_load_music(slot, std.string.toStringz(fileName)) < 0)
    {
      throw new SDLInitFailedException("Couldn't load: " ~ fileName);
    }
  }

  public void loadChunk(string name, int ch)
  {
    string fileName = chunksDir ~ name;
    if (sound_load_chunk(slot, std.string.toStringz(fileName), ch) < 0)
    {
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
    sound_play_music(slot);
  }

  public void playChunk()
  {
    sound_play_chunk(slot);
  }
}
