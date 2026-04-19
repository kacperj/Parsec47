/*
 * $Id: Field.d,v 1.4 2004/01/01 11:26:41 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.Field;

private:
import abagames.util.Vector;
import abagames.util.BoxCollision;

extern(C) {
  uint field_create_ring_display_list();
  void field_init(Box b);
  void field_set_aim_z(float z);
  void field_set_aim_speed(float speed);
  void field_set_color(int mode);
  void field_move();
  void field_set_type(int type_);
  void field_draw();
  bool field_check_hit(float px, float py);
  bool field_check_hit_with_space(float px, float py, float space);
  Box field_get_collision_box();
}

/**
 * Stage field.
 */
public class Field
{
public:
  static const int TYPE_NUM = 4;
  static bool noField = false;

  @property Box box() { return field_get_collision_box(); }

  public void init(Box collisionBox)
  {
    field_init(collisionBox);
  }

  public void setAimZ(float z)
  {
    field_set_aim_z(z);
  }

  public void setAimSpeed(float speed)
  {
    field_set_aim_speed(speed);
  }

  public void setColor(int mode)
  {
    field_set_color(mode);
  }

  public void move()
  {
    field_move();
  }

  public void setType(int type)
  {
    field_set_type(type);
  }

  public void draw()
  {
    if (noField)
      return;
    field_draw();
  }

  public bool checkHit(Vector p)
  {
    return field_check_hit(p.x, p.y);
  }

  public bool checkHit(Vector p, float space)
  {
    return field_check_hit_with_space(p.x, p.y, space);
  }

  public static void createDisplayLists()
  {
    field_create_ring_display_list();
  }
}
