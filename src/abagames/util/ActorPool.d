/*
 * $Id: ActorPool.d,v 1.2 2004/01/01 11:26:43 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.util.ActorPool;

private:
import abagames.util.Actor;

/**
 * Object pooling for actors.
 */
public class ActorPool(T : Actor)
{
public:
  T[] actor;
protected:
  int actorIdx;

  public this(int n, T delegate() factory)
  {
    actor = new T[n];
    for (int i = 0; i < actor.length; i++)
    {
      actor[i] = factory();
      actor[i].isExist = false;
    }
    actorIdx = n;
  }

  public T getInstance()
  {
    for (int i = 0; i < actor.length; i++)
    {
      actorIdx--;
      if (actorIdx < 0)
        actorIdx = cast(int)(actor.length - 1);
      
      if (!actor[actorIdx].isExist)
        return actor[actorIdx];
    }
    return null;
  }

  public void move()
  {
    foreach (T a; actor)
    {
      if (a.isExist)
        a.move();
    }
  }

  public void draw()
  {
    foreach (T a; actor)
    {
      if (a.isExist)
        a.draw();
    }
  }

  public void clear()
  {
    foreach (T a; actor)
    {
      a.isExist = false;
    }
  }
}
