module abagames.p47.Effects;

import abagames.util.Vector;


private extern (C) {
  void particles_init_new(float x, float y, float deg, float ofs, float speed);
  void fragments_init_new(float x1, float y1, float x2, float y2, float z, float speed, float deg);
}

public static class Effects
{
  public static void addParticle(Vector pos, float deg, float ofs, float speed)
  {
    particles_init_new(pos.x, pos.y, deg, ofs, speed);
  }

  public static void addFragments(int n, float x1, float y1, float x2, float y2, float z,
    float speed, float deg)
  {
    for (int i = 0; i < n; i++)
    {
      fragments_init_new(x1, y1, x2, y2, z, speed, deg);
    }
  }
}