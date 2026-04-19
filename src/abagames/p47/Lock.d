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
  void lock_draw(int state, int lockAnimProgress,
                 float lockedPosX, float lockedPosY,
                 Vector2* laserTrace);
}

private:
import std.math;
import abagames.util.Vector;
import abagames.util.Actor;
import abagames.util.Rand;
import abagames.util.BoxCollision;
import abagames.p47.Enemy;
import abagames.p47.SoundManager;
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
  
  static const int NO_COLLISION_CNT = 8;

  float lockMinY;
  Enemy lockedEnemy;
  int lockedPart;

  bool released;

private:
  static Rand rand;
  Vector2 vel;
  Vector2 lockedPos;
  static const int LENGTH = 12;
  Vector2[LENGTH] laserTrace;
  int lockAnimProgress;
  private static const float SPEED = 0.01;
  private static const int LOCK_ANIM_DURATION = 8;

  public static void init()
  {
    rand = new Rand;
  }

  private void reset()
  {
    for (int i = 0; i < LENGTH; i++)
    {
      laserTrace[i] = Vector2(ship_get_pos_x(), ship_get_pos_y());
    }
    vel.x = rand.nextSignedFloat(1.5);
    vel.y = -2;
    lockAnimProgress = 0;
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
    lockAnimProgress = 0;
  }

  public Vector2 getLaserHead() {
    return laserTrace[0];
  }

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
      if (lockAnimProgress >= LOCK_ANIM_DURATION)
      {
        state = LOCKED;
        SoundManager.playSe(SoundManager.LASER);
        lockAnimProgress = 0;
      }
      break;
    case LOCKED:
      if (lockAnimProgress >= NO_COLLISION_CNT)
        state = FIRED;
      goto case;
    case FIRED:
      goto case;
    case CANCELED:
      if (state != CANCELED)
      {
        Vector2 directionToTarget = lockedPos - laserTrace[0];

        if (isLockLost())
        {
          state = CANCELED;
        }
        else
        {
          Vector2 speedCorrection = directionToTarget * SPEED;

          vel = vel + speedCorrection;
        }
        vel = vel * 0.9;

        laserTrace[0] = laserTrace[0] + (directionToTarget * 0.002 * lockAnimProgress);
      }
      else
      {
        vel.y += (field_get_collision_box().halfHeight() * 2 - laserTrace[0].y) * SPEED;
      }

      for (int i = LENGTH - 1; i > 0; i--)
      {
        laserTrace[i] = laserTrace[i - 1];
      }
      laserTrace[0] = laserTrace[0] + vel;

      if (laserTrace[0].y > field_get_collision_box().halfHeight() + 5)
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
      float d = atan2(laserTrace[1].x - laserTrace[0].x, laserTrace[1].y - laserTrace[0].y);
      Effects.addParticle(laserTrace[0], d, 0, SPEED * 32);
      break;
    case HIT:
      for (int i = 1; i < LENGTH; i++)
      {
        laserTrace[i] = laserTrace[i - 1];
      }
      if (lockAnimProgress > 5)
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
    lockAnimProgress++;
  }

  public override void draw()
  {
    lock_draw(state, lockAnimProgress, lockedPos.x, lockedPos.y, laserTrace.ptr);
  }

  private bool isLockLost() 
  {
    return !lockedEnemy.isExist ||
          lockedEnemy.shield <= 0 ||
          (lockedPart >= 0 && lockedEnemy.battery[lockedPart].shield <= 0)
  }
}

