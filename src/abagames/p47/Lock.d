/*
 * $Id: Lock.d,v 1.1 2004/01/01 11:26:42 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.Lock;

private extern (C) {
  float ship_get_pos_x();
  float ship_get_pos_y();
  Box field_get_collision_box();
}

private:
import std.math;
import abagames.util.Vector;
import abagames.util.Actor;
import abagames.util.Rand;
import abagames.util.BoxCollision;
import abagames.p47.Enemy;
import abagames.p47.SoundManager;
import abagames.p47.Renderer;
import abagames.p47.Effects;

/**
 * Lock laser.
 */
public class Lock : Actor
{
public:
  static enum
  {
    SEARCH,
    SEARCHED,
    LOCKING,
    LOCKED,
    FIRED,
    HIT,
    CANCELED
  }
  int state;
  static const int LENGTH = 12;
  Vector[LENGTH] pos;
  static const int NO_COLLISION_CNT = 8;
  int cnt;
  float lockMinY;
  Enemy lockedEnemy;
  int lockedPart;
  Vector lockedPos;
  bool released;
private:
  static Rand rand;
  Vector vel;

  public static void init()
  {
    rand = new Rand;
  }

  public this()
  {
    for (int i = 0; i < LENGTH; i++)
    {
      pos[i] = new Vector;
    }
    vel = new Vector;
    lockedPos = new Vector;
  }

  private void reset()
  {
    for (int i = 0; i < LENGTH; i++)
    {
      pos[i].x = ship_get_pos_x();
      pos[i].y = ship_get_pos_y();
    }
    vel.x = rand.nextSignedFloat(1.5);
    vel.y = -2;
    cnt = 0;
  }

  public void set()
  {
    reset();
    state = SEARCH;
    lockMinY = field_get_collision_box().halfHeight() * 2;
    released = false;
    isExist = true;
  }

  public void hit()
  {
    state = HIT;
    cnt = 0;
  }

  private static const float SPEED = 0.01;
  private static const int LOCK_CNT = 8;
  private static const Color LOCK_COLOR = Color(1.0, 0.8, 0.5, 1);

  public override void move()
  {
    if (state == SEARCH)
    {
      isExist = false;
      return;
    }
    else if (state == SEARCHED)
    {
      state = LOCKING;
      SoundManager.playSe(SoundManager.LOCK);
    }
    if (state != HIT && state != CANCELED)
    {
      if (lockedPart < 0)
      {
        lockedPos.x = lockedEnemy.pos.x;
        lockedPos.y = lockedEnemy.pos.y;
      }
      else
      {
        lockedPos.x = lockedEnemy.pos.x + lockedEnemy.type.batteryType[lockedPart].collisionPos.x;
        lockedPos.y = lockedEnemy.pos.y + lockedEnemy.type.batteryType[lockedPart].collisionPos.y;
      }
    }
    switch (state)
    {
    case LOCKING:
      if (cnt >= LOCK_CNT)
      {
        state = LOCKED;
        SoundManager.playSe(SoundManager.LASER);
        cnt = 0;
      }
      break;
    case LOCKED:
      if (cnt >= NO_COLLISION_CNT)
        state = FIRED;
      goto case;
    case FIRED:
      goto case;
    case CANCELED:
      if (state != CANCELED)
      {
        if (!lockedEnemy.isExist ||
          lockedEnemy.shield <= 0 ||
          (lockedPart >= 0 && lockedEnemy.battery[lockedPart].shield <= 0))
        {
          state = CANCELED;
        }
        else
        {
          vel.x += (lockedPos.x - pos[0].x) * SPEED;
          vel.y += (lockedPos.y - pos[0].y) * SPEED;
        }
        vel.x *= 0.9;
        vel.y *= 0.9;
        pos[0].x += (lockedPos.x - pos[0].x) * 0.002 * cnt;
        pos[0].y += (lockedPos.y - pos[0].y) * 0.002 * cnt;
      }
      else
      {
        vel.y += (field_get_collision_box().halfHeight() * 2 - pos[0].y) * SPEED;
      }
      for (int i = LENGTH - 1; i > 0; i--)
      {
        pos[i].x = pos[i - 1].x;
        pos[i].y = pos[i - 1].y;
      }
      pos[0].x += vel.x;
      pos[0].y += vel.y;
      if (pos[0].y > field_get_collision_box().halfHeight() + 5)
      {
        if (state == CANCELED)
        {
          isExist = false;
          return;
        }
        else
        {
          state = LOCKED;
          SoundManager.playSe(SoundManager.LASER);
          reset();
        }
      }
      float d = atan2(pos[1].x - pos[0].x, pos[1].y - pos[0].y);
      Effects.addParticle(pos[0], d, 0, SPEED * 32);
      break;
    case HIT:
      for (int i = 1; i < LENGTH; i++)
      {
        pos[i].x = pos[i - 1].x;
        pos[i].y = pos[i - 1].y;
      }
      if (cnt > 5)
      {
        if (!released)
        {
          state = LOCKED;
          SoundManager.playSe(SoundManager.LASER);
          reset();
        }
        else
        {
          isExist = false;
          return;
        }
      }
      break;
    default:
      break;
    }
    cnt++;
  }

  public override void draw()
  {
    switch (state)
    {
    case LOCKING:
      float y = lockedPos.y - (LOCK_CNT - cnt) * 0.5;
      float d = (LOCK_CNT - cnt) * 0.1;
      float r = (LOCK_CNT - cnt) * 0.5 + 0.8;

      RetroParam param = RetroParam((LOCK_CNT - cnt) / LOCK_CNT, 0.2);

      for (int i = 0; i < 3; i++, d += 6.28 / 3)
      {
        Renderer.drawBoxRetro(lockedPos.x + sin(d) * r,
          y + cos(d) * r,
          0.2, 1, d + 3.14 / 2, 
          LOCK_COLOR,
          param);
      }
      break;
    case LOCKED:
    case FIRED:
    case CANCELED:
    case HIT:
      float d = 0;
      float r = 0.8;

      RetroParam param = RetroParam(0, 0.2);
      for (int i = 0; i < 3; i++, d += 6.28 / 3)
      {
        Renderer.drawBoxRetro(lockedPos.x + sin(d) * r,
          lockedPos.y + cos(d) * r,
          0.2, 1, d + 3.14 / 2,
          LOCK_COLOR,
          param);
      }
      r = cnt * 0.1;
      for (int i = 0; i < LENGTH - 1; i++, r -= 0.1)
      {
        float rr = r;
        if (rr < 0)
          rr = 0;
        else if (rr > 1)
          rr = 1;
        param = RetroParam(rr, 0.33);

        Renderer.drawLineRetro(pos[i].x, pos[i].y, pos[i + 1].x, pos[i + 1].y, 0, LOCK_COLOR, param);
      }
      break;
    default:
      break;
    }
  }
}

