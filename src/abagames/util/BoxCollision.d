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
}

