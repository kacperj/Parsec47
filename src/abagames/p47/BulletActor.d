/*
 * $Id: BulletActor.d,v 1.5 2004/01/01 11:26:41 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.BulletActor;

private:
import std.math;
import bulletml;
import abagames.util.Actor;
import abagames.util.Vector;
import abagames.p47.Bullet;
import abagames.p47.Field;
import abagames.p47.BulletActorPool;
import abagames.p47.Ship;

private extern (C) {
  void bullet_actor_draw_retro(float d, float rt, float bulletSize, int shape, int color);
  void bullet_actor_draw(int shape, int color, float deg, float xReverse, int cnt,
                         float posX, float posY, float rtCnt, float bulletSize);
}

public extern (C) {
  void bullet_actor_create_display_lists();
}

/**
 * Actor of the bullet.
 */
public class BulletActor : Actor
{
public:
  Bullet bullet;
  static float totalBulletsSpeed;
private:
  static const float FIELD_SPACE = 0.5;
  static int BULLET_DISAPPEAR_CNT = 180;
  Field field;
  Ship ship;
  static int nextId;
  bool isSimple;
  bool isTop;
  bool isVisible;
  BulletMLParser* parser;
  Vector ppos;
  const float SHIP_HIT_WIDTH = 0.2;
  int cnt;
  const float RETRO_CNT = 24;
  float rtCnt;
  bool shouldBeRemoved;
  bool backToRetro;

  public static void init()
  {
    nextId = 0;
  }

  public static void resetTotalBulletsSpeed()
  {
    totalBulletsSpeed = 0;
  }

  public this(Field field, Ship ship)
  {
    this.field = field;
    this.ship = ship;
    bullet = new Bullet(nextId);
    ppos = new Vector;
    nextId++;
  }

  private void start(float speedRank, int shape, int color, float size, float xReverse)
  {
    isExist = true;
    isTop = false;
    isVisible = true;
    ppos.x = bullet.pos.x;
    ppos.y = bullet.pos.y;
    bullet.setParam(speedRank, shape, color, size, xReverse);
    cnt = 0;
    rtCnt = 0;
    shouldBeRemoved = false;
    backToRetro = false;
  }

  public void set(BulletMLRunner* runner,
    float x, float y, float deg, float speed, float rank,
    float speedRank, int shape, int color, float size, float xReverse)
  {
    bullet.set(runner, x, y, deg, speed, rank);
    bullet.isMorph = false;
    isSimple = false;
    start(speedRank, shape, color, size, xReverse);
  }

  public void set(BulletMLRunner* runner,
    float x, float y, float deg, float speed, float rank,
    float speedRank, int shape, int color, float size, float xReverse,
    BulletMLParser*[] morph, int morphNum, int morphIdx, int morphCnt)
  {
    bullet.set(runner, x, y, deg, speed, rank);
    bullet.setMorph(morph, morphNum, morphIdx, morphCnt);
    isSimple = false;
    start(speedRank, shape, color, size, xReverse);
  }

  public void set(float x, float y, float deg, float speed, float rank,
    float speedRank, int shape, int color, float size, float xReverse)
  {
    bullet.set(x, y, deg, speed, rank);
    bullet.isMorph = false;
    isSimple = true;
    start(speedRank, shape, color, size, xReverse);
  }

  public void setInvisible()
  {
    isVisible = false;
  }

  public void setTop(BulletMLParser* parser)
  {
    this.parser = parser;
    isTop = true;
    setInvisible();
  }

  public void rewind()
  {
    bullet.remove();
    BulletMLRunner* runner = BulletMLRunner_new_parser(parser);
    BulletActorPool.registFunctions(runner);
    bullet.setRunner(runner);
    bullet.resetMorph();
  }

  public void remove()
  {
    shouldBeRemoved = true;
  }

  private void removeForced()
  {
    if (!isSimple)
      bullet.remove();
    isExist = false;
  }

  public void toRetro()
  {
    if (!isVisible || backToRetro)
      return;
    backToRetro = true;
    if (rtCnt >= RETRO_CNT)
      rtCnt = RETRO_CNT - 0.1;
  }

  // Check if the bullet hits the ship.
  private void checkShipHit()
  {
    float bmvx, bmvy, inaa;
    bmvx = ppos.x;
    bmvy = ppos.y;
    bmvx -= bullet.pos.x;
    bmvy -= bullet.pos.y;
    inaa = bmvx * bmvx + bmvy * bmvy;
    if (inaa > 0.00001)
    {
      float sofsx, sofsy, inab, hd;
      sofsx = ship.pos.x;
      sofsy = ship.pos.y;
      sofsx -= bullet.pos.x;
      sofsy -= bullet.pos.y;
      inab = bmvx * sofsx + bmvy * sofsy;
      if (inab >= 0 && inab <= inaa)
      {
        hd = sofsx * sofsx + sofsy * sofsy - inab * inab / inaa;
        if (hd >= 0 && hd <= SHIP_HIT_WIDTH)
        {
          ship.destroyed();
        }
      }
    }
  }

  public override void move()
  {
    ppos.x = bullet.pos.x;
    ppos.y = bullet.pos.y;
    if (!isSimple)
    {
      bullet.move();
      if (isTop && bullet.isEnd())
        rewind();
    }
    if (shouldBeRemoved)
    {
      removeForced();
      return;
    }
    float sr;
    if (rtCnt < RETRO_CNT)
    {
      sr = bullet.speedRank * (0.3 + (rtCnt / RETRO_CNT) * 0.7);
      if (backToRetro)
      {
        rtCnt -= sr;
        if (rtCnt <= 0)
        {
          removeForced();
          return;
        }
      }
      else
      {
        rtCnt += sr;
      }
      if (ship.cnt < -Ship.INVINCIBLE_CNT / 2 && isVisible && rtCnt >= RETRO_CNT)
      {
        removeForced();
        return;
      }
    }
    else
    {
      sr = bullet.speedRank;
      if (cnt > BULLET_DISAPPEAR_CNT)
        toRetro();
    }
    bullet.pos.x +=
      (sin(bullet.deg) * bullet.speed + bullet.acc.x) * sr * bullet.xReverse;
    bullet.pos.y +=
      (cos(bullet.deg) * bullet.speed - bullet.acc.y) * sr;
    if (isVisible)
    {
      totalBulletsSpeed += bullet.speed * sr;
      if (rtCnt > RETRO_CNT)
        checkShipHit();
      if (field_check_hit_with_space(bullet.pos.x, bullet.pos.y, FIELD_SPACE))
        removeForced();
    }
    cnt++;
  }

  public override void draw()
  {
    if (!isVisible)
      return;
    bullet_actor_draw(bullet.shape, bullet.color, bullet.deg, bullet.xReverse,
      cnt, bullet.pos.x, bullet.pos.y, rtCnt, bullet.bulletSize);
  }

}

