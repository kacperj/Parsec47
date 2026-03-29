module abagames.p47.BonusState;

public extern (C)
{
  void bonus_state_reset();
}

public class BonusState
{
  public static void resetBonusScore()
  {
    bonus_state_reset();
  }
}