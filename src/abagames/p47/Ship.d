/*
 * $Id: Ship.d,v 1.4 2004/01/01 11:26:42 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.Ship;

private extern (C) {
  void ship_set_pos(float x, float y);
  void ship_set_cnt(int cnt);
  void bonus_state_reset();
  void ship_create_display_lists();
  void ship_draw(int cnt, float bank, float fireWideDeg, int ttlCnt);
  void rolls_init_new();
  void rolls_release_all();
  void shots_init_new(float x, float y, float deg, float bx1, float by1, float bx2, float by2);
}

private:
import std.math;
import abagames.util.Vector;
import abagames.util.Rand;
import abagames.util.sdl.Pad;
import abagames.p47.Bullet;
import abagames.p47.Field;
import abagames.p47.P47GameManager;
import abagames.p47.SoundManager;
import abagames.p47.Effects;
import abagames.p47.ShipMode;

/**
 * My ship.
 */
public class Ship
{
public:
  static bool isSlow = false;
  Vector pos;
  const float SIZE = 0.3;
  static const int RESTART_CNT = 300;
  static const int INVINCIBLE_CNT = 228;
  int cnt;
private:
  static Rand _rand;
  static @property Rand rand()
  {
    if (!_rand)
      _rand = new Rand;
    return _rand;
  }

  Pad pad;
  Field field;
  P47GameManager manager;
  Vector ppos;
  const float BASE_SPEED = 0.6;
  const float SLOW_BASE_SPEED = 0.3;
  float baseSpeed, slowSpeed;

  float speed;
  Vector vel;
  const float BANK_BASE = 50;
  float bank;
  Vector firePos;
  float fireWideDeg;
  const float FIRE_WIDE_BASE_DEG = 0.7;
  const float FIRE_NARROW_BASE_DEG = 0.5;
  int fireCnt;
  const float TURRET_INTERVAL_LENGTH = 0.2;
  int ttlCnt;
  const float FIELD_SPACE = 1.5;
  float fieldLimitX, fieldLimitY;
  int rollLockCnt;
  bool rollCharged;
  int mode;

  public void init(Pad pad, Field field, P47GameManager manager)
  {
    this.pad = pad;
    this.field = field;
    this.manager = manager;
    pos = new Vector;
    ppos = new Vector;
    vel = new Vector;
    firePos = new Vector;
    ttlCnt = 0;
    fieldLimitX = field.box.halfWidth() - FIELD_SPACE;
    fieldLimitY = field.box.halfHeight() - FIELD_SPACE;
  }

  public void start(int mode)
  {
    this.mode = mode; 
    ppos.x = pos.x = 0;
    ppos.y = pos.y = -field.box.halfHeight() / 2;
    vel.x = vel.y = 0;
    speed = BASE_SPEED;
    fireWideDeg = FIRE_WIDE_BASE_DEG;
    cnt = -INVINCIBLE_CNT;
    fireCnt = 0;
    rollLockCnt = 0;
    bank = 0;
    rollCharged = false;
    bonus_state_reset();
  }

  public void setSpeedRate(float rate)
  {
    if (!isSlow)
      baseSpeed = BASE_SPEED * rate;
    else
      baseSpeed = BASE_SPEED * 0.7;
    slowSpeed = SLOW_BASE_SPEED * rate;
  }

  public void destroyed()
  {
    if (cnt <= 0)
      return;
    SoundManager.playSe(SoundManager.SHIP_DESTROYED);
    if (mode == ShipMode.ROLL)
      rolls_release_all();
    else
      manager.releaseLock();

    manager.shipDestroyed();
    Effects.addFragments(30, pos.x, pos.y, pos.x, pos.y, 0, 0.08, std.math.PI);
    for (int i = 0; i < 45; i++)
      Effects.addParticle(pos, rand.nextFloat(std.math.PI * 2), 0, 0.6);
    start(mode);
    cnt = -RESTART_CNT;
  }

  public void move()
  {
    cnt++;
    if (cnt < -INVINCIBLE_CNT)
    {
      return;
    }

    if (pad.isButton2())
    {
      speed += (slowSpeed - speed) * 0.2;
      fireWideDeg += (FIRE_NARROW_BASE_DEG - fireWideDeg) * 0.1;
      rollLockCnt++;
      if (mode == ShipMode.ROLL)
      {
        if (rollLockCnt % 15 == 0)
        {
          rolls_init_new();
          SoundManager.playSe(SoundManager.ROLL_CHARGE);
          rollCharged = true;
        }
      }
      else
      {
        if (rollLockCnt % 10 == 0)
        {
          manager.addLock();
        }
      }
    }
    else
    {
      speed += (baseSpeed - speed) * 0.2;
      fireWideDeg += (FIRE_WIDE_BASE_DEG - fireWideDeg) * 0.1;
      if (mode == ShipMode.ROLL)
      {
        if (rollCharged)
        {
          rollLockCnt = 0;
          rolls_release_all();
          SoundManager.playSe(SoundManager.ROLL_RELEASE);
          rollCharged = false;
        }
      }
      else
      {
        rollLockCnt = 0;
        manager.releaseLock();
      }
    }
    vel.x = vel.y = 0;
    if (pad.isPadUp())
      vel.y = speed;
    else if (pad.isPadDown())
      vel.y = -speed;
    if (pad.isPadRight())
      vel.x = speed;
    else if (pad.isPadLeft())
      vel.x = -speed;
    if (vel.x != 0 && vel.y != 0)
    {
      vel.x *= 0.707;
      vel.y *= 0.707;
    }
    ppos.x = pos.x;
    ppos.y = pos.y;
    pos.x += vel.x;
    pos.y += vel.y;
    bank += (vel.x * BANK_BASE - bank) * 0.1;
    if (pos.x < -fieldLimitX)
      pos.x = -fieldLimitX;
    else if (pos.x > fieldLimitX)
      pos.x = fieldLimitX;
    if (pos.y < -fieldLimitY)
      pos.y = -fieldLimitY;
    else if (pos.y > fieldLimitY)
      pos.y = fieldLimitY;
    if (pad.isButton1())
    {
      float td;
      switch (fireCnt % 4)
      {
      case 0:
        firePos.x = pos.x + TURRET_INTERVAL_LENGTH;
        firePos.y = pos.y;
        td = 0;
        break;
      case 1:
        firePos.x = pos.x + TURRET_INTERVAL_LENGTH;
        firePos.y = pos.y;
        td = fireWideDeg * (fireCnt / 4 % 5) * 0.2;
        break;
      case 2:
        firePos.x = pos.x - TURRET_INTERVAL_LENGTH;
        firePos.y = pos.y;
        td = 0;
        break;
      case 3:
        firePos.x = pos.x - TURRET_INTERVAL_LENGTH;
        firePos.y = pos.y;
        td = -fireWideDeg * (fireCnt / 4 % 5) * 0.2;
        break;
      default:
        break;
      }
      shots_init_new(firePos.x, firePos.y, td, field.box.x1, field.box.y1, field.box.x2, field.box.y2);
      SoundManager.playSe(SoundManager.SHOT);
      fireCnt++;
    }
    Bullet.target.x = pos.x;
    Bullet.target.y = pos.y;
    ttlCnt++;
    ship_set_pos(pos.x, pos.y);
    ship_set_cnt(cnt);
  }

  public void draw()
  {
    ship_draw(cnt, bank, fireWideDeg, ttlCnt);
  }

  public static void createDisplayLists()
  {
    ship_create_display_lists();
  }
}
