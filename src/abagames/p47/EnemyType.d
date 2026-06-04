/*
 * $Id: EnemyType.d,v 1.4 2004/01/01 11:26:41 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.EnemyType;

private:
import std.math;
import bulletml;
import abagames.util.Vector;
import abagames.util.Rand;
import abagames.p47.BarrageManager;
import abagames.p47.Bullet;
import abagames.p47.Renderer;
import abagames.p47.ShipMode;


extern(C) Color create_enemy_color(int variant);


/**
 * Barrage pattern.
 */
public class Barrage
{
public:
  BulletMLParser* parser;
  BulletMLParser*[Bullet.MORPH_MAX] morphParser;
  int morphNum, morphCnt;
  float rank, speedRank, morphRank;
  int shape, color;
  float bulletSize;
  float xReverse;
}


/**
 * Enemys' wing with batteries.
 */
public class BatteryType
{
public:
  static const int WING_SHAPE_POINT_NUM = 3;
  static const int WING_BATTERY_MAX = 3;
  static const int BARRAGE_PATTERN_MAX = 8;
  Vector2[WING_SHAPE_POINT_NUM] wingShapePos;
  Vector collisionPos, collisionSize;
  Vector[WING_BATTERY_MAX] batteryPos;
  int batteryNum;
  float b;  
  float g;
  float r;
  Barrage[BARRAGE_PATTERN_MAX] barrage;
  bool xReverseAlternate;
  int shield;

  public this()
  {
    for (int i = 0; i < BARRAGE_PATTERN_MAX; i++)
    {
      barrage[i] = new Barrage;
    }
    collisionPos = new Vector;
    collisionSize = new Vector;
    for (int i = 0; i < WING_BATTERY_MAX; i++)
    {
      batteryPos[i] = new Vector;
    }
  }
}

/**
 * Enemys' specifications.
 */
public class EnemyType
{
public:
  static const int BARRAGE_PATTERN_MAX = BatteryType.BARRAGE_PATTERN_MAX;
  static const int BODY_SHAPE_POINT_NUM = 4;
  static const int BATTERY_MAX = 4;

  Barrage[BARRAGE_PATTERN_MAX] barrage;
  Vector2[BODY_SHAPE_POINT_NUM] bodyShapePos;
  Vector collisionSize;
  bool wingCollision;
  Color enemyColor;
  float retroSize;
  BatteryType[BATTERY_MAX] batteryType;
  int batteryNum;
  int shield;
  int fireInterval, firePeriod, barragePatternNum;
  int id;
  static enum
  {
    SMALL,
    MIDDLE,
    LARGE,
    MIDDLEBOSS,
    LARGEBOSS,
  }
  int type;
  // To avoid the forward reference.

private:
  static Rand rand;
  static BarrageManager barrageManager;

  public static void init(BarrageManager manager)
  {
    rand = new Rand;
    barrageManager = manager;
  }

  public this(int newId)
  {
    collisionSize = new Vector;
    for (int i = 0; i < BARRAGE_PATTERN_MAX; i++)
    {
      barrage[i] = new Barrage;
    }
    for (int i = 0; i < BATTERY_MAX; i++)
    {
      batteryType[i] = new BatteryType;
    }

    id = newId;
  }

  // To avoid using the same morph pattern.
  private static bool[BarrageManager.BARRAGE_MAX] usedMorphParser;

  private void setBarrageType(Barrage br, int btn, int mode)
  {
    int barrageTypeRandom = rand.nextInt();

    br.parser = barrageManager.getMoveParser(btn, barrageTypeRandom);

    for (int i = 0; i < BarrageManager.BARRAGE_MAX; i++)
      usedMorphParser[i] = false;
    
    int morphParserCategory = (mode == ShipMode.ROLL) ? BarrageManager.MORPH : BarrageManager.MORPH_LOCK;

    int availableMorphParsers = barrageManager.getParserNumbersForCategory(morphParserCategory);

    for (int i = 0; i < br.morphParser.length; i++)
    {
      int mi = getUnusedMorphIndex(availableMorphParsers);
      usedMorphParser[mi] = true;

      br.morphParser[i] = barrageManager.getMoveParser(morphParserCategory, mi);
    }
    br.morphNum = br.morphParser.length;
  }

  private int getUnusedMorphIndex(int availableMorphParsers)
  {
    int mi = rand.nextInt(availableMorphParsers);

    for (int j = 0; j < availableMorphParsers; j++)
    {
      if (!usedMorphParser[mi])
          break;
      mi++;
      if (mi >= availableMorphParsers)
        mi = 0;
    }
    return mi;
  }

  // Barrage intense.
  enum
  {
    NORMAL,
    WEAK,
    VERYWEAK,
    MORPHWEAK
  }

  private void setBarrageRank(Barrage br, float rank, int intense, int mode)
  {
    if (rank <= 0)
    {
      br.rank = 0;
      return;
    }
    br.rank = sqrt(rank) / (8 - rand.nextInt(3));
    if (br.rank > 0.8)
      br.rank = rand.nextFloat(0.2) + 0.8;
    rank /= (br.rank + 2);
    if (intense == WEAK)
      br.rank /= 2;
    if (mode == ShipMode.ROLL)
      br.speedRank = sqrt(rank) * (rand.nextFloat(0.2) + 1);
    else
      br.speedRank = sqrt(rank * 0.66) * (rand.nextFloat(0.2) + 0.8);
    if (br.speedRank < 1)
      br.speedRank = 1;
    if (br.speedRank > 2)
      br.speedRank = sqrt(br.speedRank) + 0.27;
    br.morphRank = rank / br.speedRank;
    br.morphCnt = 0;
    while (br.morphRank > 1)
    {
      br.morphCnt++;
      br.morphRank /= 3;
    }
    if (intense == VERYWEAK)
    {
      br.morphRank /= 2;
      br.morphCnt /= 1.7f;
    }
    else if (intense == MORPHWEAK)
    {
      br.morphRank /= 2;
    }
    else if (intense == WEAK)
    {
      br.morphRank /= 1.5f;
    }
  }

  private void setBarrageRankSlow(Barrage br, float rank, int intense, int mode, float slow)
  {
    setBarrageRank(br, rank, intense, mode);
    br.speedRank *= slow;
  }

  public static const int BULLET_SHAPE_NUM = 7;
  public static const int BULLET_COLOR_NUM = 4;

  private void setBarrageShape(Barrage br, float size)
  {
    // To avoid the forward reference.
    br.shape = rand.nextInt(BULLET_SHAPE_NUM);
    br.color = rand.nextInt(BULLET_COLOR_NUM);
    br.bulletSize = (1.0 + rand.nextSignedFloat(0.1)) * size;
  }

  private int getEnemyColorType()
  {
    return rand.nextInt(3);
  }

  private Color createEnemyColor(int variant)
  {
    return create_enemy_color(variant);
  }

  private static const float[][] enemySizes =
    [
      [0.3, 0.3, 0.3, 0.1, 0.1, 1.0, 0.4, 0.6, 0.9],
      [0.4, 0.2, 0.4, 0.1, 0.15, 2.2, 0.2, 1.6, 1.0],
      [0.6, 0.3, 0.5, 0.1, 0.2, 3.0, 0.3, 1.4, 1.2],
      [0.9, 0.3, 0.7, 0.2, 0.25, 5.0, 0.6, 3.0, 1.5],
      [1.2, 0.2, 0.9, 0.1, 0.3, 7.0, 0.8, 4.5, 1.5],
    ];
  // Set the shepe of the BatteryType.
  private void setEnemyShapeAndWings(int size)
  {
    int colorType = getEnemyColorType();
    enemyColor = createEnemyColor(colorType);

    const float[] enemySize = EnemyType.enemySizes[size];

    float x1 = enemySize[0] + rand.nextSignedFloat(enemySize[1]);
    float y1 = enemySize[2] + rand.nextSignedFloat(enemySize[3]);
    float x2 = enemySize[0] + rand.nextSignedFloat(enemySize[1]);
    float y2 = enemySize[2] + rand.nextSignedFloat(enemySize[3]);

    bodyShapePos = [
      Vector2(-x1, y1), 
      Vector2(x1, y1), 
      Vector2(x2, -y2), 
      Vector2(-x2, -y2)
    ];
    
    retroSize = enemySize[4];
    switch (size)
    {
    case SMALL:
    case MIDDLE:
    case MIDDLEBOSS:
      batteryNum = 2;
      break;
    case LARGE:
    case LARGEBOSS:
      batteryNum = 4;
      break;
    default:
      break;
    }
    float px, py, mpx, mpy;
    int bsl;
    if (x1 > x2)
      collisionSize.x = x1;
    else
      collisionSize.x = x2;
    if (y1 > y2)
      collisionSize.y = y1;
    else
      collisionSize.y = y2;


    Color batteryColor;
    
    for (int i = 0; i < batteryNum; i++)
    {
      BatteryType bt = batteryType[i];
      int wrl = 1;

      if (i % 2 == 0)
      {
        px = enemySize[5] + rand.nextFloat(enemySize[6]);
        if (batteryNum <= 2)
        {
          py = rand.nextSignedFloat(enemySize[7]);
        }
        else
        {
          if (i < 2)
          {
            py = rand.nextFloat(enemySize[7] / 2) + enemySize[7] / 2;
          }
          else
          {
            py = -rand.nextFloat(enemySize[7] / 2) - enemySize[7] / 2;
          }
        }
        float md;
        if (rand.nextInt(2) == 0)
          md = rand.nextFloat(std.math.PI / 2) - std.math.PI / 4;
        else
          md = rand.nextFloat(std.math.PI / 2) + std.math.PI / 4 * 3;
        mpx = px / 2 + sin(md) * (enemySize[8] / 2 + rand.nextFloat(enemySize[8] / 2));
        mpy = py / 2 + cos(md) * (enemySize[8] / 2 + rand.nextFloat(enemySize[8] / 2));
        switch (size)
        {
        case SMALL:
        case MIDDLE:
        case LARGE:
          bsl = 1;
          break;
        case MIDDLEBOSS:
          bsl = 150 + rand.nextInt(30);
          break;
        case LARGEBOSS:
          bsl = 200 + rand.nextInt(50);
          break;
        default:
          break;
        }
        batteryColor = createEnemyColor(colorType);
        wrl = -1;
        if (!wingCollision)
        {
          if (px > collisionSize.x)
            collisionSize.x = px;
          float cpy = fabs(py);
          if (cpy > collisionSize.y)
            collisionSize.y = cpy;
          cpy = fabs(mpy);
          if (cpy > collisionSize.y)
            collisionSize.y = cpy;
        }
      }
      bt.wingShapePos = createWings(px, py, mpx, mpy, wrl);

      bt.collisionPos.x = (px + px / 4) / 2 * wrl;
      bt.collisionPos.y = (py + mpy + py / 4) / 3;
      bt.collisionSize.x = px / 4 * 3 / 2;
      float sy1 = fabs(py - mpy) / 2;
      float sy2 = fabs(py - py / 4) / 2;
      if (sy1 > sy2)
        bt.collisionSize.y = sy1;
      else
        bt.collisionSize.y = sy2;
      bt.r = batteryColor.r;
      bt.g = batteryColor.g;
      bt.b = batteryColor.b;
      bt.shield = bsl;
    }
  }

  private static Vector2[BatteryType.WING_SHAPE_POINT_NUM] createWings(float px, float py, float mpx, float mpy, int wrl)
  {
    return [
      Vector2(px / 4 * wrl, py / 4), 
      Vector2(px * wrl, py), 
      Vector2(mpx * wrl, mpy)
    ];
  }

  // Set the barrage of the BatteryType.
  private void setBattery(float rank, int n, int barrageType, int barrageIntense,
    int idx, int ptnIdx, float slow, int mode)
  {
    BatteryType bt = batteryType[idx];
    BatteryType bt2 = batteryType[idx + 1];
    Barrage br = bt.barrage[ptnIdx];
    Barrage br2 = bt2.barrage[ptnIdx];
    setBarrageType(br, barrageType, mode);
    setBarrageRankSlow(br, rank / n, barrageIntense, mode, slow);
    setBarrageShape(br, 0.8);
    br.xReverse = rand.nextInt(2) * 2 - 1;
    br2.parser = br.parser;
    for (int i = 0; i < Bullet.MORPH_MAX; i++)
    {
      br2.morphParser[i] = br.morphParser[i];
    }
    br2.morphNum = br.morphNum;
    br2.morphCnt = br.morphCnt;
    br2.rank = br.rank;
    br2.speedRank = br.speedRank;
    br2.morphRank = br.morphRank;
    br2.shape = br.shape;
    br2.color = br.color;
    br2.bulletSize = br.bulletSize;
    br2.xReverse = -br.xReverse;
    if (rand.nextInt(4) == 0)
      bt.xReverseAlternate = bt2.xReverseAlternate = true;
    else
      bt.xReverseAlternate = bt2.xReverseAlternate = false;
    float px = bt.wingShapePos[1].x, py = bt.wingShapePos[1].y;
    float mpx = bt.wingShapePos[2].x, mpy = bt.wingShapePos[2].y;
    for (int i = 0; i < n; i++)
    {
      bt.batteryPos[i].x = px;
      bt.batteryPos[i].y = py;
      bt2.batteryPos[i].x = -px;
      bt2.batteryPos[i].y = py;
      px += (mpx - px) / (n - 1);
      py += (mpy - py) / (n - 1);
    }
    bt.batteryNum = bt2.batteryNum = n;
  }

  public void setSmallEnemyType(float rank, int mode)
  {
    type = SMALL;
    barragePatternNum = 1;
    wingCollision = false;
    Barrage br = barrage[0];
    if (mode == ShipMode.ROLL)
      setBarrageType(br, BarrageManager.SMALL, mode);
    else
      setBarrageType(br, BarrageManager.SMALL_LOCK, mode);
    setBarrageRank(br, rank, VERYWEAK, mode);
    setBarrageShape(br, 0.7);
    br.xReverse = rand.nextInt(2) * 2 - 1;
    setEnemyShapeAndWings(SMALL);
    setBattery(0, 0, 0, NORMAL, 0, 0, 1, mode);
    shield = 1;
    fireInterval = 99999;
    firePeriod = 150 + rand.nextInt(40);
    if (rank < 10)
      firePeriod /= (2 - rank * 0.1);
  }

  public void setMiddleEnemyType(float rank, int mode)
  {
    type = MIDDLE;
    barragePatternNum = 1;
    wingCollision = false;
    Barrage br = barrage[0];
    setBarrageType(br, BarrageManager.MIDDLE, mode);
    float cr, sr;
    if (mode == ShipMode.ROLL)
    {
      switch (rand.nextInt(6))
      {
      case 0:
      case 1:
        cr = rank / 3 * 2;
        sr = 0;
        break;
      case 2:
        cr = rank / 4;
        sr = rank / 4;
        break;
      case 3:
      case 4:
      case 5:
        cr = 0;
        sr = rank / 2;
        break;
      default:
        break;
      }
    }
    else
    {
      switch (rand.nextInt(6))
      {
      case 0:
      case 1:
        cr = rank / 5;
        sr = rank / 4;
        break;
      case 2:
      case 3:
      case 4:
      case 5:
        cr = 0;
        sr = rank / 2;
        break;
      default:
        break;
      }
    }
    setBarrageRank(br, cr, MORPHWEAK, mode);
    setBarrageShape(br, 0.75);
    br.xReverse = rand.nextInt(2) * 2 - 1;
    setEnemyShapeAndWings(MIDDLE);
    if (mode == ShipMode.ROLL)
    {
      shield = 40 + rand.nextInt(10);
      setBattery(sr, 1, BarrageManager.MIDDLESUB, NORMAL, 0, 0, 1, mode);
      fireInterval = 100 + rand.nextInt(60);
      firePeriod = cast(int)(fireInterval / (1.8 + rand.nextFloat(0.7)));
    }
    else
    {
      shield = 30 + rand.nextInt(8);
      setBattery(sr, 1, BarrageManager.MIDDLESUB_LOCK, NORMAL, 0, 0, 1, mode);
      fireInterval = 72 + rand.nextInt(30);
      firePeriod = cast(int)(fireInterval / (1.2 + rand.nextFloat(0.2)));
    }
    if (rank < 10)
      firePeriod /= (2 - rank * 0.1);
  }

  public void setLargeEnemyType(float rank, int mode)
  {
    type = LARGE;
    barragePatternNum = 1;
    wingCollision = false;
    Barrage br = barrage[0];
    setBarrageType(br, BarrageManager.LARGE, mode);
    float cr, sr1, sr2;
    if (mode == ShipMode.ROLL)
    {
      switch (rand.nextInt(9))
      {
      case 0:
      case 1:
      case 2:
      case 3:
        cr = rank;
        sr1 = sr2 = 0;
        break;
      case 4:
        cr = rank / 3 * 2;
        sr1 = rank / 3 * 2;
        sr2 = 0;
        break;
      case 5:
        cr = rank / 3 * 2;
        sr1 = 0;
        sr2 = rank / 3 * 2;
        break;
      case 6:
      case 7:
      case 8:
        cr = 0;
        sr1 = rank / 3 * 2;
        sr2 = rank / 3 * 2;
        break;
      default:
        break;
      }
    }
    else
    {
      switch (rand.nextInt(9))
      {
      case 0:
        cr = rank / 4 * 3;
        sr1 = sr2 = 0;
        break;
      case 1:
      case 2:
        cr = rank / 4 * 2;
        sr1 = rank / 3 * 2;
        sr2 = 0;
        break;
      case 3:
      case 4:
        cr = rank / 4 * 2;
        sr1 = 0;
        sr2 = rank / 3 * 2;
        break;
      case 5:
      case 6:
      case 7:
      case 8:
        cr = 0;
        sr1 = rank / 3 * 2;
        sr2 = rank / 3 * 2;
        break;
      default:
        break;
      }
    }
    setBarrageRank(br, cr, WEAK, mode);
    setBarrageShape(br, 0.8);
    br.xReverse = rand.nextInt(2) * 2 - 1;
    setEnemyShapeAndWings(LARGE);
    if (mode == ShipMode.ROLL)
    {
      shield = 60 + rand.nextInt(10);
      setBattery(sr1, 1, BarrageManager.MIDDLESUB, NORMAL, 0, 0, 1, mode);
      setBattery(sr2, 1, BarrageManager.MIDDLESUB, NORMAL, 2, 0, 1, mode);
      fireInterval = 150 + rand.nextInt(60);
      firePeriod = cast(int)(fireInterval / (1.3 + rand.nextFloat(0.8)));
    }
    else
    {
      shield = 45 + rand.nextInt(8);
      setBattery(sr1, 1, BarrageManager.MIDDLESUB_LOCK, NORMAL, 0, 0, 1, mode);
      setBattery(sr2, 1, BarrageManager.MIDDLESUB_LOCK, NORMAL, 2, 0, 1, mode);
      fireInterval = 100 + rand.nextInt(50);
      firePeriod = cast(int)(fireInterval / (1.2 + rand.nextFloat(0.2)));
    }
    if (rank < 10)
      firePeriod /= (2 - rank * 0.1);
  }

  public void setMiddleBossEnemyType(float rank, int mode)
  {
    type = MIDDLEBOSS;
    barragePatternNum = 2 + rand.nextInt(2);
    wingCollision = true;
    int bn = 1 + rand.nextInt(2);
    for (int i = 0; i < barragePatternNum; i++)
    {
      Barrage br = barrage[i];
      setBarrageType(br, BarrageManager.LARGE, mode);
      float cr, sr;
      switch (rand.nextInt(3))
      {
      case 0:
        cr = rank;
        sr = 0;
        break;
      case 1:
        cr = rank / 3;
        sr = rank / 3;
        break;
      case 2:
        cr = 0;
        sr = rank;
        break;
      default:
        break;
      }
      setBarrageRankSlow(br, cr, NORMAL, mode, 0.9);
      setBarrageShape(br, 0.9);
      br.xReverse = rand.nextInt(2) * 2 - 1;
      setEnemyShapeAndWings(MIDDLEBOSS);
      setBattery(sr, bn, BarrageManager.MIDDLE, WEAK, 0, i, 0.9, mode);
    }
    shield = 300 + rand.nextInt(50);
    fireInterval = 200 + rand.nextInt(40);
    firePeriod = cast(int)(fireInterval / (1.2 + rand.nextFloat(0.4)));
    if (rank < 10)
      firePeriod /= (2 - rank * 0.1);
  }

  public void setLargeBossEnemyType(float rank, int mode)
  {
    type = LARGEBOSS;
    barragePatternNum = 2 + rand.nextInt(3);
    wingCollision = true;
    int bn1 = 1 + rand.nextInt(3);
    int bn2 = 1 + rand.nextInt(3);
    for (int i = 0; i < barragePatternNum; i++)
    {
      Barrage br = barrage[i];
      setBarrageType(br, BarrageManager.LARGE, mode);
      float cr, sr1, sr2;
      switch (rand.nextInt(3))
      {
      case 0:
        cr = rank;
        sr1 = sr2 = 0;
        break;
      case 1:
        cr = rank / 3;
        sr1 = rank / 3;
        sr2 = 0;
        break;
      case 2:
        cr = rank / 3;
        sr1 = 0;
        sr2 = rank / 3;
        break;
      default:
        break;
      }
      setBarrageRankSlow(br, cr, NORMAL, mode, 0.9);
      setBarrageShape(br, 1.0);
      br.xReverse = rand.nextInt(2) * 2 - 1;
      setEnemyShapeAndWings(LARGEBOSS);
      setBattery(sr1, bn1, BarrageManager.MIDDLE, NORMAL, 0, i, 0.9, mode);
      setBattery(sr2, bn2, BarrageManager.MIDDLE, NORMAL, 2, i, 0.9, mode);
    }
    shield = 400 + rand.nextInt(50);
    fireInterval = 220 + rand.nextInt(60);
    firePeriod = cast(int)(fireInterval / (1.2 + rand.nextFloat(0.3)));
    if (rank < 10)
      firePeriod /= (2 - rank * 0.1);
  }
}
