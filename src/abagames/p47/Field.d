/*
 * $Id: Field.d,v 1.4 2004/01/01 11:26:41 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.Field;

private:
import std.math;
import opengl;
import abagames.util.Vector;
import abagames.p47.Renderer;
import abagames.util.BoxCollision;

extern(C) uint field_create_ring_display_list();

// Mode constants (mirrored from P47GameManager to avoid cyclic import)
private enum : int
{
  ROLL = 0,
  LOCK = 1
}

/**
 * Stage field.
 */
public class Field
{
public:
  static const int TYPE_NUM = 4;
  Box box;
  float eyeZ;
  float aimZ;
  float aimSpeed;
private:
  static int displayListIdx;
  static const int RING_NUM = 16;
  static const float RING_ANGLE_INT = 10;
  float roll, yaw;
  float z;
  float speed;
  float yawYBase, yawZBase;
  float aimYawYBase, aimYawZBase;
  Color color;

  public void init()
  {
    box = Box.createWithHalfExtents(11, 16);
    eyeZ = 20;
    roll = yaw = 0;
    z = aimZ = 10;
    speed = aimSpeed = 0.1;
    yawYBase = yawZBase = 0;
  }

  public void setColor(int mode)
  {
    switch (mode)
    {
    case ROLL:
      color = Color(0.2, 0.2, 0.7, 0.7);
      break;
    case LOCK:
      color = Color(0.5, 0.3, 0.6, 0.7);
      break;
    default:
      break;
    }
  }

  public void move()
  {
    roll += speed;
    if (roll >= RING_ANGLE_INT)
      roll -= RING_ANGLE_INT;
    yaw += speed;
    z += (aimZ - z) * 0.003;
    speed += (aimSpeed - speed) * 0.004;
    yawYBase += (aimYawYBase - yawYBase) * 0.002;
    yawZBase += (aimYawZBase - yawZBase) * 0.002;
  }

  public void setType(int type)
  {
    switch (type)
    {
    case 0:
      aimYawYBase = 30;
      aimYawZBase = 0;
      break;
    case 1:
      aimYawYBase = 0;
      aimYawZBase = 20;
      break;
    case 2:
      aimYawYBase = 50;
      aimYawZBase = 10;
      break;
    case 3:
      aimYawYBase = 10;
      aimYawZBase = 30;
      break;
    default:
      break;
    }
  }

  public void draw()
  {
    Renderer.setColor(color);
    float d = -RING_NUM * RING_ANGLE_INT / 2 + roll;
    for (int i = 0; i < RING_NUM; i++)
    {
      for (int j = 1; j < 8; j++)
      {
        float sc = cast(float) j / 16 + 0.5;
        glPushMatrix();
        glTranslatef(0, 0, z);
        glRotatef(d, 1, 0, 0);
        glRotatef(sin(yaw / 180 * PI) * yawYBase, 0, 1, 0);
        glRotatef(sin(yaw / 180 * PI) * yawZBase, 0, 0, 1);
        glScalef(1, 1, sc);
        glCallList(displayListIdx);
        glPopMatrix();
      }
      d += RING_ANGLE_INT;
    }
  }

  public bool checkHit(Vector p)
  {
    return box.checkHit(p.x, p.y);
  }

  public bool checkHit(Vector p, float space)
  {
    return box.checkHit(p.x, p.y, space);
  }

  public static void createDisplayLists()
  {
    displayListIdx = field_create_ring_display_list();
  }
}
