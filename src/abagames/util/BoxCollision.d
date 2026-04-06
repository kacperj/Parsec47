module abagames.util.BoxCollision;

import std.math;
import abagames.util.Vector;


/**
 * Axis-aligned box centered at the origin, defined by half-extents.
 */
public struct Box {
  float x1 = 0;
  float y1 = 0;
  float x2 = 0;
  float y2 = 0;

  static Box createWithHalfExtents(float halfWidth, float halfHeight) {
    Box box;
    box.x1 = -halfWidth;
    box.y1 = -halfHeight;
    box.x2 = halfWidth;
    box.y2 = halfHeight;
    return box;
  }

  float width() { return fabs(x2 - x1); }
  float height() { return fabs(y2 - y1); }

  float halfWidth() { return width() * 0.5; }
  float halfHeight() { return height() * 0.5; }

  public bool checkHit(Vector point) {
    return BoxCollision.checkHit(this, point.x, point.y);
  }

  public bool checkHit(Vector2 point) {
    return BoxCollision.checkHit(this, point.x, point.y);
  }


  public bool checkHit(Vector point, float space) {
    return BoxCollision.checkHit(this, point.x, point.y, space);
  }

  public bool checkHit(Vector2 point, float space) {
    return BoxCollision.checkHit(this, point.x, point.y, space);
  }

  public bool checkHit(float x, float y) {
    return BoxCollision.checkHit(this, x, y);
  }

  public bool checkHit(float x, float y, float space) {
    return BoxCollision.checkHit(this, x, y, space);
  }
}

public static class BoxCollision
{
  /**
  * Returns true if point (px, py) is outside the box.
  */
  public static bool checkHit(Box box, float px, float py) {
    return px < box.x1 || px > box.x2 || py < box.y1 || py > box.y2;
  }

  /**
  * Returns true if point (px, py) is outside the box shrunk by space on each side.
  */
  public static bool checkHit(Box box, float px, float py, float space) {
    return px < box.x1 + space || px > box.x2 - space
        || py < box.y1 + space || py > box.y2 - space;
  }
}

