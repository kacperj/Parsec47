module abagames.util.Rand;

private:
import core.time;

extern (C)
{
  void rand_set_seed(uint s);
  int rand_next_int(int n);
  float rand_next_float(float n);
  float rand_next_signed_float(float n);
}

public class Rand
{

  public this()
  {
    rand_set_seed(cast(uint) MonoTime.currTime.ticks);
  }

  public void setSeed(long n)
  {
    rand_set_seed(cast(uint) n);
  }

  public int nextInt(int n)
  {
    return rand_next_int(n);
  }

  public float nextFloat(float n)
  {
    return rand_next_float(n);
  }

  public float nextSignedFloat(float n)
  {
    return rand_next_signed_float(n);
  }
}
