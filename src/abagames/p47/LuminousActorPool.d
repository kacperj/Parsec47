/*
 * $Id: LuminousActorPool.d,v 1.2 2004/01/01 11:26:42 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.LuminousActorPool;

private:
import abagames.util.ActorPool;
import abagames.p47.LuminousActor;

/**
 * Actor pool for the LuminousActor.
 */
public class LuminousActorPool : ActorPool!LuminousActor
{
  public this(int n, LuminousActor delegate() factory)
  {
    super(n, factory);
  }

  public void drawLuminous()
  {
    for (int i = 0; i < actor.length; i++)
    {
      if (actor[i].isExist)
        actor[i].drawLuminous();
    }
  }
}
