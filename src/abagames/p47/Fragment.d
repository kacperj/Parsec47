/*
 * $Id: Fragment.d,v 1.2 2004/01/01 11:26:42 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.Fragment;

private:
import std.math;
import opengl;
import abagames.util.Vector;
import abagames.util.Rand;
import abagames.util.Actor;
import abagames.p47.Renderer;
import abagames.p47.LuminousActor;
import abagames.p47.Field;

/**
 * Enemys' fragments.
 */
public class Fragment : LuminousActor
{
public:
  static const Color FRAGMENT_COLOR = Color(1, 0.8, 0.6, 1);
private:
  static Rand rand;
  static const int POINT_NUM = 2;
  Vector[POINT_NUM] pos;
  Vector[POINT_NUM] vel;
  Vector impact;
  float z;
  float lumAlp;
  float retro;
  int cnt;

  public static this()
  {
    rand = new Rand;
  }

  public this()
  {
    for (int i = 0; i < POINT_NUM; i++)
    {
      pos[i] = new Vector;
      vel[i] = new Vector;
    }
    impact = new Vector;
  }

  public void set(float x1, float y1, float x2, float y2, float z, float speed, float deg)
  {
    float r1 = rand.nextFloat(1);
    float r2 = rand.nextFloat(1);
    pos[0].x = x1 * r1 + x2 * (1 - r1);
    pos[0].y = y1 * r1 + y2 * (1 - r1);
    pos[1].x = x1 * r2 + x2 * (1 - r2);
    pos[1].y = y1 * r2 + y2 * (1 - r2);
    for (int i = 0; i < POINT_NUM; i++)
    {
      vel[i].x = rand.nextSignedFloat(1) * speed;
      vel[i].y = rand.nextSignedFloat(1) * speed;
    }
    impact.x = sin(deg) * speed * 4;
    impact.y = cos(deg) * speed * 4;
    this.z = z;
    cnt = 32 + rand.nextInt(24);
    lumAlp = 0.8 + rand.nextFloat(0.2);
    retro = 1;
    isExist = true;
  }

  public override void move()
  {
    cnt--;
    if (cnt < 0)
    {
      isExist = false;
      return;
    }
    for (int i = 0; i < POINT_NUM; i++)
    {
      pos[i].add(vel[i]);
      pos[i].add(impact);
      vel[i].mul(0.98);
    }
    impact.mul(0.95);
    lumAlp *= 0.98;
    retro *= 0.97;
  }

  public override void draw()
  {
    Renderer.drawLineRetro(pos[0].x, pos[0].y, pos[1].x, pos[1].y, z, FRAGMENT_COLOR, RetroParam(retro, 0.2));
  }

  public override void drawLuminous()
  {
    if (lumAlp < 0.2)
      return;
    Renderer.setColor(Color(FRAGMENT_COLOR.r, FRAGMENT_COLOR.g, FRAGMENT_COLOR.b, lumAlp));
    glVertex3f(pos[0].x, pos[0].y, z);
    glVertex3f(pos[1].x, pos[1].y, z);
  }
}
