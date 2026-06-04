/*
 * $Id: StageManager.d,v 1.4 2004/01/01 11:26:42 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.StageManager;

private extern (C) {
  int stage_get_appearance_count_for_section(int section, int middleRushSectionNum,
                                             int mode, int enemyType);
}

private:
import std.math;
import bulletml;
import abagames.util.Rand;
import abagames.util.Vector;
import abagames.p47.BarrageManager;
import abagames.p47.P47GameManager;
import abagames.p47.Field;
import abagames.p47.Enemy;
import abagames.p47.EnemyType;
import abagames.p47.EnemyTypeTracker;
import abagames.p47.SoundManager;

/**
 * Manage the stage data(enemies' appearance).
 */
public class StageManager
{
  // Appearance point.
  static enum
  {
    TOP,
    SIDE,
    BACK
  }
  // Appearance pattern.
  static enum
  {
    ONE_SIDE,
    ALTERNATE,
    BOTH_SIDES
  }
  // Appearance position is fixed or not.
  static enum
  {
    RANDOM,
    FIXED
  }
  // Enemy type.
  static enum
  {
    SMALL,
    MIDDLE,
    LARGE,
  }

  private struct EnemyAppearance
  {
  public:
    EnemyType type;
    int point, pattern, sequence;
    float pos;
    int num, interval, groupInterval;
    int cnt, left, side;
    int moveType;
    int moveTypeRandom;
  }

  private struct SpawnPoint
  {
    Vector2 pos;
    float dir;
  }

public:
  int parsec;
  bool bossSection;
private:
  static Rand rand;

  P47GameManager gameManager;
  Field field;
  const int SIMULTANEOUS_APPEARNCE_MAX = 4;
  EnemyAppearance[SIMULTANEOUS_APPEARNCE_MAX] appearance;
  const int SMALL_ENEMY_TYPE_MAX = 3;
  EnemyType[SMALL_ENEMY_TYPE_MAX] smallType;
  const int MIDDLE_ENEMY_TYPE_MAX = 4;
  EnemyType[MIDDLE_ENEMY_TYPE_MAX] middleType;
  const int LARGE_ENEMY_TYPE_MAX = 2;
  EnemyType[LARGE_ENEMY_TYPE_MAX] largeType;
  EnemyType middleBossType;
  EnemyType largeBossType;
  int apNum;
  int sectionCnt, sectionIntervalCnt, section;
  float rank, rankInc;
  int middleRushSectionNum;
  bool middleRushSection;
  int stageType;

  public void init(P47GameManager gm, Field f)
  {
    gameManager = gm;
    field = f;
    rand = new Rand;

    int typeId = 0;

    for (int i = 0; i < smallType.length; i++) {
      smallType[i] = new EnemyType(typeId);
      typeId++;
    }
      
    for (int i = 0; i < middleType.length; i++) {
      middleType[i] = new EnemyType(typeId);
      typeId++;
    }

    for (int i = 0; i < largeType.length; i++) {
      largeType[i] = new EnemyType(typeId);
      typeId++;
    }
      
    middleBossType = new EnemyType(typeId);
    typeId++;
    largeBossType = new EnemyType(typeId);
  }

  private void createEnemyData()
  {
    for (int i = 0; i < smallType.length; i++)
      smallType[i].setSmallEnemyType(rank, gameManager.mode);
    for (int i = 0; i < middleType.length; i++)
      middleType[i].setMiddleEnemyType(rank, gameManager.mode);
    for (int i = 0; i < largeType.length; i++)
      largeType[i].setLargeEnemyType(rank, gameManager.mode);
    middleBossType.setMiddleBossEnemyType(rank, gameManager.mode);
    largeBossType.setLargeBossEnemyType(rank, gameManager.mode);
  }

  private static EnemyAppearance setAppearancePattern(EnemyAppearance ap)
  {
    switch (rand.nextInt(5))
    {
    case 0:
      ap.pattern = ONE_SIDE;
      break;
    case 1:
    case 2:
      ap.pattern = ALTERNATE;
      break;
    case 3:
    case 4:
      ap.pattern = BOTH_SIDES;
      break;
    default:
      break;
    }
    switch (rand.nextInt(3))
    {
    case 0:
      ap.sequence = RANDOM;
      break;
    case 1:
    case 2:
      ap.sequence = FIXED;
      break;
    default:
      break;
    }

    return ap;
  }

  private EnemyAppearance setSmallAppearance()
  {
    EnemyAppearance ap;

    ap.type = smallType[rand.nextInt(smallType.length)];
    if (rand.nextFloat(1) > 0.2)
    {
      ap.point = TOP;
      ap.moveType = BarrageManager.SMALLMOVE;
    }
    else
    {
      ap.point = SIDE;
      ap.moveType = BarrageManager.SMALLSIDEMOVE;
    }

    ap = setAppearancePattern(ap);
    if (ap.pattern == ONE_SIDE)
      ap.pattern = ALTERNATE;
    switch (rand.nextInt(4))
    {
    case 0:
      ap.num = 7 + rand.nextInt(5);
      ap.groupInterval = 72 + rand.nextInt(15);
      ap.interval = 15 + rand.nextInt(5);
      break;
    case 1:
      ap.num = 5 + rand.nextInt(3);
      ap.groupInterval = 56 + rand.nextInt(10);
      ap.interval = 20 + rand.nextInt(5);
      break;
    case 2:
    case 3:
      ap.num = 2 + rand.nextInt(2);
      ap.groupInterval = 45 + rand.nextInt(20);
      ap.interval = 25 + rand.nextInt(5);
      break;
    default:
      break;
    }

    return ap;
  }

  private EnemyAppearance setMiddleAppearance()
  {
    EnemyAppearance ap;

    ap.type = middleType[rand.nextInt(middleType.length)];

    ap.point = TOP;
    ap.moveType = BarrageManager.MIDDLEMOVE;
    ap = setAppearancePattern(ap);
    switch (rand.nextInt(3))
    {
    case 0:
      ap.num = 4;
      ap.groupInterval = 240 + rand.nextInt(150);
      ap.interval = 80 + rand.nextInt(30);
      break;
    case 1:
      ap.num = 2;
      ap.groupInterval = 180 + rand.nextInt(60);
      ap.interval = 180 + rand.nextInt(20);
      break;
    case 2:
      ap.num = 1;
      ap.groupInterval = 150 + rand.nextInt(50);
      ap.interval = 100;
      break;
    default:
      break;
    }

    return ap;
  }

  private EnemyAppearance setLargeAppearance()
  {
    EnemyAppearance ap;

    ap.type = largeType[rand.nextInt(largeType.length)];
    int mt;
    ap.point = TOP;
    ap.moveType = BarrageManager.LARGEMOVE;
    ap = setAppearancePattern(ap);
    switch (rand.nextInt(3))
    {
    case 0:
      ap.num = 3;
      ap.groupInterval = 400 + rand.nextInt(100);
      ap.interval = 240 + rand.nextInt(40);
      break;
    case 1:
      ap.num = 2;
      ap.groupInterval = 400 + rand.nextInt(60);
      ap.interval = 300 + rand.nextInt(20);
      break;
    case 2:
      ap.num = 1;
      ap.groupInterval = 270 + rand.nextInt(50);
      ap.interval = 200;
      break;
    default:
      break;
    }

    return ap;
  }

  private EnemyAppearance createAppearance(int type)
  {
    EnemyAppearance ap;

    switch (type)
    {
    case SMALL:
      ap = setSmallAppearance();
      break;
    case MIDDLE:
      ap = setMiddleAppearance();
      break;
    case LARGE:
      ap = setLargeAppearance();
      break;
    default:
      break;
    }
    ap.cnt = 0;
    ap.left = ap.num;
    ap.side = rand.nextInt(2) * 2 - 1;
    ap.pos = rand.nextFloat(1);

    return ap;
  }

  private void createSectionData()
  {
    apNum = 0;
    if (rank <= 0)
      return;
    field_set_aim_speed(0.1 + section * 0.02);
    if (section == 4)
    {
      // Set the middle boss.
      Vector pos = new Vector;
      pos.x = 0;
      pos.y = field.box.halfHeight() / 4 * 3;
      gameManager.addBoss(pos, std.math.PI, middleBossType);
      bossSection = true;
      sectionIntervalCnt = sectionCnt = 2 * 60;
      field.setAimZ(11);
      return;
    }
    else if (section == 9)
    {
      // Set the large boss.
      Vector pos = new Vector;
      pos.x = 0;
      pos.y = field.box.halfHeight() / 4 * 3;
      gameManager.addBoss(pos, std.math.PI, largeBossType);
      bossSection = true;
      sectionIntervalCnt = sectionCnt = 3 * 60;
      field.setAimZ(12);
      return;
    }
    else if (section == middleRushSectionNum)
    {
      // In this section, no small enemy.
      middleRushSection = true;
      field.setAimZ(9);
    }
    else
    {
      middleRushSection = false;
      field.setAimZ(10 + rand.nextSignedFloat(0.3));
    }
    bossSection = false;
    if (section == 3)
      sectionIntervalCnt = 2 * 60;
    else if (section == 3)
      sectionIntervalCnt = 4 * 60;
    else
      sectionIntervalCnt = 1 * 60;
    sectionCnt = sectionIntervalCnt + 10 * 60;

    int[3] enemyTypes = [SMALL, MIDDLE, LARGE];

    foreach (int enemyType; enemyTypes)
    {
      int numberOfEnemyType = stage_get_appearance_count_for_section(section, middleRushSectionNum, gameManager.mode, enemyType);

      for (int i = 0; i < numberOfEnemyType; i++)
      {
        EnemyAppearance ap = createAppearance(enemyType);

        ap.moveTypeRandom = rand.nextInt();

        appearance[apNum] = ap;
        apNum++;
      }
    }
  }

  private void createStage()
  {
    createEnemyData();
    middleRushSectionNum = 2 + rand.nextInt(6);
    if (middleRushSectionNum <= 4)
      middleRushSectionNum++;
    field_set_type(stageType % Field.TYPE_NUM);
    sound_manager_play_bgm(stageType % SoundManager.BGM_NUM);
    stageType++;
  }

  private void gotoNextSection()
  {
    section++;
    parsec++;
    if (gameManager.state == P47GameManager.TITLE && section >= 4)
    {
      section = 0;
      parsec -= 4;
    }
    if (section >= 10)
    {
      section = 0;
      rank += rankInc;
      createStage();
    }
    createSectionData();
  }

  public void setRank(float baseRank, float inc, int startParsec, int type)
  {
    rank = baseRank;
    rankInc = inc;
    rank += rankInc * (startParsec / 10);
    section = -1;
    parsec = startParsec - 1;
    stageType = type;
    createStage();
    gotoNextSection();
  }

  private SpawnPoint processAppearance(EnemyAppearance* ap)
  {
    Vector2 apos;
    float p;
    switch (ap.sequence)
    {
    case RANDOM:
      p = rand.nextFloat(1);
      break;
    case FIXED:
      p = ap.pos;
      break;
    default:
      break;
    }
    float d;
    switch (ap.point)
    {
    case TOP:
      switch (ap.pattern)
      {
      case BOTH_SIDES:
        apos.x = (p - 0.5) * field.box.halfWidth() * 1.8;
        break;
      default:
        apos.x = (p * 0.6 + 0.2) * field.box.halfWidth() * ap.side;
        break;
      }
      apos.y = field.box.halfHeight() - Enemy.FIELD_SPACE;
      d = std.math.PI;
      break;
    case BACK:
      switch (ap.pattern)
      {
      case BOTH_SIDES:
        apos.x = (p - 0.5) * field.box.halfWidth() * 1.8;
        break;
      default:
        apos.x = (p * 0.6 + 0.2) * field.box.halfWidth() * ap.side;
        break;
      }
      apos.y = -field.box.halfHeight() + Enemy.FIELD_SPACE;
      d = 0;
      break;
    case SIDE:
      switch (ap.pattern)
      {
      case BOTH_SIDES:
        apos.x = (field.box.halfWidth() - Enemy.FIELD_SPACE) * (rand.nextInt(2) * 2 - 1);
        break;
      default:
        apos.x = (field.box.halfWidth() - Enemy.FIELD_SPACE) * ap.side;
        break;
      }
      apos.y = (p * 0.4 + 0.4) * field.box.halfHeight();
      if (apos.x < 0)
        d = std.math.PI / 2;
      else
        d = std.math.PI / 2 * 3;
      break;
    default:
      break;
    }
    apos.x *= 0.88;
    ap.left--;
    if (ap.left <= 0)
    {
      ap.cnt = ap.groupInterval;
      ap.left = ap.num;
      if (ap.pattern != ONE_SIDE)
        ap.side *= -1;
      ap.pos = rand.nextFloat(1);
    }
    else
    {
      ap.cnt = ap.interval;
    }
    return SpawnPoint(apos, d);
  }

  public void move()
  {
    for (int i = 0; i < apNum; i++)
    {
      EnemyAppearance* ap = &(appearance[i]);
      ap.cnt--;
      if (ap.cnt > 0)
      {
        // Add the extra enemy.
        if (!middleRushSection)
        {
          if (ap.type.type == EnemyType.SMALL && !EnemyTypeTracker.exists(ap.type.id))
          {
            ap.cnt = 0;
            EnemyTypeTracker.mark(ap.type.id);
          }
        }
        else
        {
          if (ap.type.type == EnemyType.MIDDLE && !EnemyTypeTracker.exists(ap.type.id))
          {
            ap.cnt = 0;
            EnemyTypeTracker.mark(ap.type.id);
          }
        }
        continue;
      }
      SpawnPoint sp = processAppearance(ap);
      gameManager.addEnemy(sp.pos, sp.dir, ap.type, ap.moveType, ap.moveTypeRandom);
    }

    if (!bossSection ||
      (!EnemyTypeTracker.exists(middleBossType.id) && !EnemyTypeTracker.exists(largeBossType.id)))
      sectionCnt--;
    
    if (sectionCnt < sectionIntervalCnt)
    {
      if (section == 9 && sectionCnt == sectionIntervalCnt - 1)
        SoundManager.fadeMusic();
      apNum = 0;
      if (sectionCnt <= 0)
        gotoNextSection();
    }
    
    EnemyTypeTracker.clear();
  }
}
