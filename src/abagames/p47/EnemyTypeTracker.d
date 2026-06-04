module abagames.p47.EnemyTypeTracker;

/**
 * Tracks which enemy types are currently alive on the field.
 * Repopulated each frame by Enemy.move().
 */
public class EnemyTypeTracker
{
  private static const int ENEMY_TYPE_MAX = 32;
  private static bool[] types = new bool[ENEMY_TYPE_MAX];

  public static bool exists(int id)
  {
    return types[id];
  }

  public static void mark(int id)
  {
    types[id] = true;
  }

  public static void clear()
  {
    types[] = false;
  }
}
