/*
 * $Id: Bonus.d,v 1.4 2004/01/01 11:26:41 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.Bonus;

private:
import std.math;
import abagames.util.Vector;
import abagames.util.Rand;
import abagames.util.Actor;
import abagames.util.BoxCollision;
import abagames.p47.Ship;


private extern (C) {
  void bonus_collected();
  void bonus_draw(float posX, float posY, int cnt, bool isDown, bool isInhaled);
  void bonus_state_reset();
  float ship_get_pos_x();
  float ship_get_pos_y();
  Box field_get_collision_box();
}

/**
 * Bonus items.
 */
public class Bonus : Actor
{
public:
  static float rate;
private:
  static const float BASE_SPEED = 0.1;
  static float speed;
  static const float INHALE_WIDTH = 3;
  static const float ACQUIRE_WIDTH = 1;
  static const int RETRO_CNT = 20;
  static Rand rand;
  float fieldLimitX, fieldLimitY;
  Ship ship;
  Vector pos;
  Vector2 vel;
  int cnt;
  bool isDown;
  bool isInhaled;
  int inhaleCnt;

  public static void init()
  {
    rand = new Rand;
  }

  public static void setSpeedRate(float r)
  {
    rate = r;
    speed = BASE_SPEED * rate;
  }

  public this(Ship ship)
  {
    this.ship = ship;
    pos = new Vector;
    fieldLimitX = field_get_collision_box().halfWidth() / 6 * 5;
    fieldLimitY = field_get_collision_box().halfHeight() / 10 * 9;
  }

  public void set(Vector p, Vector ofs)
  {
    pos.x = p.x;
    pos.y = p.y;
    if (ofs)
    {
      pos.x += ofs.x;
      pos.y += ofs.y;
    }
    vel.x = rand.nextSignedFloat(0.07);
    vel.y = rand.nextSignedFloat(0.07);
    cnt = 0;
    inhaleCnt = 0;
    isDown = true;
    isInhaled = false;
    isExist = true;
  }

  public override void move()
  {
    pos.x += vel.x;
    pos.y += vel.y;
    vel.x -= vel.x / 50;
    if (pos.x > fieldLimitX)
    {
      pos.x = fieldLimitX;
      if (vel.x > 0)
        vel.x = -vel.x;
    }
    else if (pos.x < -fieldLimitX)
    {
      pos.x = -fieldLimitX;
      if (vel.x < 0)
        vel.x = -vel.x;
    }
    if (isDown)
    {
      vel.y += (-speed - vel.y) / 50;
      if (pos.y < -fieldLimitY)
      {
        isDown = false;
        pos.y = -fieldLimitY;
        vel.y = speed;
      }
    }
    else
    {
      vel.y += (speed - vel.y) / 50;
      if (pos.y > fieldLimitY)
      {
        bonus_state_reset();
        isExist = false;
        return;
      }
    }
    cnt++;
    if (cnt < RETRO_CNT)
      return;
    float _ax = fabs(pos.x - ship_get_pos_x());
    float _ay = fabs(pos.y - ship_get_pos_y());
    float d = (_ax > _ay) ? _ax + _ay / 2 : _ay + _ax / 2;
    if (d < ACQUIRE_WIDTH * (1 + cast(float) inhaleCnt * 0.2) && ship.cnt >= -Ship.INVINCIBLE_CNT)
    {
      bonus_collected();
      isExist = false;
      return;
    }
    if (isInhaled)
    {
      inhaleCnt++;
      float ip = (INHALE_WIDTH - d) / 48;
      if (ip < 0.025)
        ip = 0.025;
      vel.x += (ship_get_pos_x() - pos.x) * ip;
      vel.y += (ship_get_pos_y() - pos.y) * ip;
      if (ship.cnt < -Ship.INVINCIBLE_CNT)
      {
        isInhaled = false;
        inhaleCnt = 0;
      }
    }
    else
    {
      if (d < INHALE_WIDTH && ship.cnt >= -Ship.INVINCIBLE_CNT)
        isInhaled = true;
    }
  }

  public override void draw()
  {
    bonus_draw(pos.x, pos.y, cnt, isDown, isInhaled);
  }
}

