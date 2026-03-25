/*
 * $Id: Vector.d,v 1.2 2004/01/01 11:26:43 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.util.Vector;

private:
import std.math;

/**
 * Vector.
 */
public class Vector
{
public:
  float x, y;

private:

  public this()
  {
  }

  public this(float x, float y)
  {
    this.x = x;
    this.y = y;
  }

  public void add(Vector v)
  {
    x += v.x;
    y += v.y;
  }

  public void mul(float a)
  {
    x *= a;
    y *= a;
  }

  public float dist(Vector v)
  {
    float ax = fabs(x - v.x);
    float ay = fabs(y - v.y);
    if (ax > ay)
    {
      return ax + ay / 2;
    }
    else
    {
      return ay + ax / 2;
    }
  }
}
