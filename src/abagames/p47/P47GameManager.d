/*
 * $Id: P47GameManager.d,v 1.6 2004/01/01 11:26:42 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.P47GameManager;

private:
import std.math;
import opengl;
import bulletml;
import abagames.util.Rand;
import abagames.util.Vector;
import abagames.util.ActorPool;
import abagames.util.sdl.MainLoop;
import abagames.util.sdl.Pad;
import abagames.p47.P47PrefManager;
import abagames.p47.P47Screen;
import abagames.p47.LetterRender;
import abagames.p47.Ship;
import abagames.p47.Field;
import abagames.p47.Enemy;
import abagames.p47.EnemyType;
import abagames.p47.BulletActor;
import abagames.p47.BulletActorPool;
import abagames.p47.BarrageManager;
import abagames.p47.Shot;  // kept for Shot.SPEED constant
import abagames.p47.Lock;
import abagames.p47.Bonus;
import abagames.p47.BonusState;
import abagames.p47.StageManager;
import abagames.p47.Title;
import abagames.p47.SoundManager;
import abagames.p47.Renderer;
import abagames.util.BoxCollision;

/**
 * Manage the game status and actor pools.
 */
private extern (C) {
  void renderer_draw_side_boards();
  void renderer_draw_side_info(int parsec);
  void renderer_draw_box(int x, int y, int w, int h);
  void renderer_draw_score();
  void sound_manager_stop_music();
  void renderer_draw_pause_status(int parsec, int pauseCnt);
  void renderer_draw_gameover_status(int parsec, int cnt);

  void particles_draw();
  void particles_draw_luminous();
  void particles_update();

  void fragments_draw();
  void fragments_draw_luminous();
  void fragments_update();

  void score_set_initial();
  void life_decrease();
  int life_get();
  int score_get();
  void score_increase(int sc);

  void shots_update();
  void shots_draw();
  void shots_clear();
  void shots_init_new(float x, float y, float deg, float bx1, float by1, float bx2, float by2);

  void rolls_clear();
  void rolls_draw();
  void rolls_init_new();
  void rolls_update();
  void rolls_release_all();
}



public class P47GameManager
{
public:
  int status;
  bool nowait = false;
  int difficulty, parsecSlot;
  static enum
  {
    ROLL,
    LOCK,
  }
  int mode;
  static enum
  {
    TITLE,
    IN_GAME,
    GAMEOVER,
    PAUSE
  }
  int state;
  MainLoop mainLoop;
  P47Screen abstScreen;
  P47PrefManager prefManager;

  public void setMainLoop(MainLoop mainLoop)
  {
    this.mainLoop = mainLoop;
  }

  public void setUIs(P47Screen screen, Pad input)
  {
    abstScreen = screen;
    this.input = input;
  }

  public void setPrefManager(P47PrefManager prefManager)
  {
    this.prefManager = prefManager;
  }

private:
  Pad input;
  Pad pad;
  const int ENEMY_MAX = 32;
  P47Screen screen;
  Rand rand;
  Field field;
  Ship ship;
  ActorPool!Enemy enemies;
  BulletActorPool bullets;
  ActorPool!Lock locks;
  ActorPool!Bonus bonuses;
  BarrageManager barrageManager;
  StageManager stageManager;
  int cnt;
  int pauseCnt;
  const int BOSS_WING_NUM = 4;
  int bossShield;
  int[BOSS_WING_NUM] bossWingShield;
  const float[P47PrefManager.MODE_NUM] SLOWDOWN_START_BULLETS_SPEED = [30, 42];
  float interval;
  Title title;

  // Initialize actor pools, load BGMs/SEs and textures.
  public void init()
  {
    pad = cast(Pad) input;
    screen = abstScreen;
    rand = new Rand;
    Field.createDisplayLists();
    field = new Field;
    field.init(Box.createWithHalfExtents(11, 16));
    Ship.createDisplayLists();
    ship = new Ship;
    ship.init(pad, field, this);
    BulletActor.createDisplayLists();
    bullets = new BulletActorPool(512, () => new BulletActor(field, ship));
    LetterRender.createDisplayLists();
    Lock.init();
    locks = new ActorPool!Lock(4, () => new Lock());
    enemies = new ActorPool!Enemy(ENEMY_MAX, () => new Enemy(field, bullets, locks, ship, this));
    Bonus.init();
    bonuses = new ActorPool!Bonus(128, () => new Bonus(ship));
    barrageManager = new BarrageManager;
    barrageManager.loadBulletMLs();
    EnemyType.init(barrageManager);
    stageManager = new StageManager;
    stageManager.init(this, barrageManager, field);
    title = new Title;
    title.init(pad, this, prefManager);
    interval = mainLoop.INTERVAL_BASE;
    SoundManager.init();
  }

  public void start()
  {
    startTitle();
  }

  public void close()
  {
    barrageManager.unloadBulletMLs();
    title.close();
    SoundManager.close();
  }

  public void addScore(int sc)
  {
    score_increase(sc);
  }

  public void shipDestroyed()
  {
    if (mode == ROLL)
      releaseRoll();
    else
      releaseLock();
    clearBullets();
    life_decrease();
    if (life_get() < 0)
      startGameover();
  }

  public void addEnemy(Vector pos, float d, EnemyType type, BulletMLParser* moveParser)
  {
    Enemy en = enemies.getInstance();
    if (!en)
      return;
    en.set(pos, d, type, moveParser);
  }

  public void clearBullets()
  {
    for (int i = 0; i < bullets.actor.length; i++)
    {
      if (!bullets.actor[i].isExist)
        continue;
      (cast(BulletActor) bullets.actor[i]).toRetro();
    }
  }

  public void addBoss(Vector pos, float d, EnemyType type)
  {
    Enemy en = enemies.getInstance();
    if (!en)
      return;
    en.setBoss(pos, d, type);
  }

  public void addShot(Vector pos, float deg)
  {
    shots_init_new(pos.x, pos.y, deg, field.box.x1, field.box.y1, field.box.x2, field.box.y2);
  }

  public void addRoll()
  {
    rolls_init_new();
  }

  public void addLock()
  {
    Lock lock = locks.getInstance();
    if (!lock)
      return;
    lock.set();
  }

  public void releaseRoll()
  {
    rolls_release_all();
  }

  public void releaseLock()
  {
    for (int i = 0; i < locks.actor.length; i++)
    {
      if (!locks.actor[i].isExist)
        continue;
      (cast(Lock) locks.actor[i]).released = true;
    }
  }

  public void addBonus(Vector pos, Vector ofs, int num)
  {
    for (int i = 0; i < num; i++)
    {
      Bonus bonus = bonuses.getInstance();
      if (!bonus)
        return;
      bonus.set(pos, ofs);
    }
  }

  public void setBossShieldMeter(int bs, int s1, int s2, int s3, int s4, float r)
  {
    r *= 0.7;
    bossShield = cast(int)(bs * r);
    bossWingShield[0] = cast(int)(s1 * r);
    bossWingShield[1] = cast(int)(s2 * r);
    bossWingShield[2] = cast(int)(s3 * r);
    bossWingShield[3] = cast(int)(s4 * r);
  }

  // Difficulty.
  public enum
  {
    PRACTICE,
    NORMAL,
    HARD,
    EXTREME,
    QUIT
  }

  public void startStage(int difficulty, int parsecSlot, int startParsec, int mode)
  {
    enemies.clear();
    bullets.clear();
    this.difficulty = difficulty;
    this.parsecSlot = parsecSlot;
    this.mode = mode;
    int stageType = rand.nextInt(99999);
    switch (difficulty)
    {
    case PRACTICE:
      stageManager.setRank(1, 4, startParsec, stageType);
      ship.setSpeedRate(0.7);
      Bonus.setSpeedRate(0.6);
      break;
    case NORMAL:
      stageManager.setRank(10, 8, startParsec, stageType);
      ship.setSpeedRate(0.9);
      Bonus.setSpeedRate(0.8);
      break;
    case HARD:
      stageManager.setRank(22, 12, startParsec, stageType);
      ship.setSpeedRate(1);
      Bonus.setSpeedRate(1);
      break;
    case EXTREME:
      stageManager.setRank(36, 16, startParsec, stageType);
      ship.setSpeedRate(1.2);
      Bonus.setSpeedRate(1.3);
      break;
    case QUIT:
      stageManager.setRank(0, 0, 0, 0);
      ship.setSpeedRate(1);
      Bonus.setSpeedRate(1);
      break;
    default:
      break;
    }
  }

  private void initShipState()
  {
    score_set_initial();
    ship.start();
  }

  private void startInGame()
  {
    state = IN_GAME;
    SoundManager.isInGame = (state == IN_GAME);
    initShipState();
    startStage(difficulty, parsecSlot, title.getStartParsec(difficulty, parsecSlot), mode);
  }

  private void startTitle()
  {
    state = TITLE;
    title.start();
    field.setColor(mode);
    initShipState();
    bullets.clear();
    ship.cnt = 0;
    startStage(difficulty, parsecSlot, title.getStartParsec(difficulty, parsecSlot), mode);
    cnt = 0;
    sound_manager_stop_music();
  }

  private void startGameover()
  {
    state = GAMEOVER;
    bonuses.clear();
    shots_clear();
    rolls_clear();
    locks.clear();
    setScreenShake(0, 0);
    interval = mainLoop.INTERVAL_BASE;
    mainLoop.interval = mainLoop.INTERVAL_BASE;
    cnt = 0;
    if (score_get() > prefManager.hiScore[mode][difficulty][parsecSlot])
      prefManager.hiScore[mode][difficulty][parsecSlot] = score_get();
    if (stageManager.parsec > prefManager.reachedParsec[mode][difficulty])
      prefManager.reachedParsec[mode][difficulty] = stageManager.parsec;
    SoundManager.fadeMusic();
  }

  private void startPause()
  {
    state = PAUSE;
    pauseCnt = 0;
  }

  private void resumePause()
  {
    state = IN_GAME;
  }

  private void stageMove()
  {
    stageManager.move();
  }

  private bool pPrsd = true;

  private void inGameMove()
  {
    stageMove();
    field.move();
    ship.move();
    bonuses.move();
    shots_update();
    enemies.move();
    if (mode == ROLL)
      rolls_update();
    else
      locks.move();
    BulletActor.resetTotalBulletsSpeed();
    bullets.move();
    particles_update();
    fragments_update();
    moveScreenShake();
    if (pad.isPausePressed())
    {
      if (!pPrsd)
      {
        pPrsd = true;
        startPause();
      }
    }
    else
    {
      pPrsd = false;
    }
    if (!nowait)
    {
      // Intentional slowdown when the total speed of bullets is over SLOWDOWN_START_BULLETS_SPEED
      if (BulletActor.totalBulletsSpeed > SLOWDOWN_START_BULLETS_SPEED[mode])
      {
        float sm = BulletActor.totalBulletsSpeed / SLOWDOWN_START_BULLETS_SPEED[mode];
        if (sm > 1.75)
          sm = 1.75;
        interval += (sm * mainLoop.INTERVAL_BASE - interval) * 0.1;
        mainLoop.interval = cast(int) interval;
      }
      else
      {
        interval += (mainLoop.INTERVAL_BASE - interval) * 0.08;
        mainLoop.interval = cast(int) interval;
      }
    }
  }

  private bool btnPrsd = true;

  private void titleMove()
  {
    title.move();
    if (cnt <= 8)
    {
      btnPrsd = true;
    }
    else
    {
      if (pad.isButton1())
      {
        if (!btnPrsd)
        {
          title.setStatus();
          if (difficulty >= P47PrefManager.DIFFICULTY_NUM)
            mainLoop.breakLoop();
          else
            startInGame();
          return;
        }
      }
      else if (pad.isButton2())
      {
        if (!btnPrsd)
        {
          title.changeMode();
          field.setColor(mode);
          btnPrsd = true;
        }
      }
      else
      {
        btnPrsd = false;
      }
    }
    stageMove();
    field.move();
    enemies.move();
    bullets.move();
  }

  private void gameoverMove()
  {
    bool gotoNextState = false;
    if (cnt <= 64)
    {
      btnPrsd = true;
    }
    else
    {
      if (pad.isButton1() || pad.isButton2())
      {
        if (!btnPrsd)
          gotoNextState = true;
      }
      else
      {
        btnPrsd = false;
      }
    }
    if (cnt > 64 && gotoNextState)
    {
      startTitle();
    }
    else if (cnt > 500)
    {
      startTitle();
    }
    field.move();
    enemies.move();
    bullets.move();
    particles_update();
    fragments_update();
  }

  private void pauseMove()
  {
    pauseCnt++;
    if (pad.isPausePressed())
    {
      if (!pPrsd)
      {
        pPrsd = true;
        resumePause();
      }
    }
    else
    {
      pPrsd = false;
    }
  }

  public void move()
  {
    if (pad.isEscapePressed())
    {
      mainLoop.breakLoop();
      return;
    }
    SoundManager.isInGame = (state == IN_GAME);
    switch (state)
    {
    case IN_GAME:
      inGameMove();
      break;
    case TITLE:
      titleMove();
      break;
    case GAMEOVER:
      gameoverMove();
      break;
    case PAUSE:
      pauseMove();
      break;
    default:
    }
    cnt++;
  }

  private void inGameDraw()
  {
    field.draw();
    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    bonuses.draw();
    glBlendFunc(GL_SRC_ALPHA, GL_ONE);
    
    glBegin(GL_LINES);
    particles_draw();
    glEnd();
    fragments_draw();
    ship.draw();
    shots_draw();
    
    if (mode == ROLL)
      rolls_draw();
    else
      locks.draw();
    enemies.draw();
    bullets.draw();
  }

  private void titleDraw()
  {
    field.draw();
    enemies.draw();
    bullets.draw();
  }

  private void gameoverDraw()
  {
    field.draw();
    glBegin(GL_LINES);
    particles_draw();
    glEnd();
    fragments_draw();
    enemies.draw();
    bullets.draw();
  }

  private void inGameDrawLuminous()
  {
    glBegin(GL_LINES);
    particles_draw_luminous();
    fragments_draw_luminous();
    glEnd();
  }

  private void drawBossShieldMeter()
  {
    renderer_draw_box(165, 6, bossShield, 6);
    int y = 24;
    for (int i = 0; i < BOSS_WING_NUM; i++)
    {
      switch (i % 2)
      {
      case 0:
        renderer_draw_box(165, y, bossWingShield[i], 6);
        break;
      case 1:
        renderer_draw_box(475 - bossWingShield[i], y, bossWingShield[i], 6);
        y += 12;
        break;
      default:
        break;
      }
    }
  }

  private void inGameDrawStatus()
  {
    renderer_draw_side_info(stageManager.parsec);
    if (stageManager.bossSection)
      drawBossShieldMeter();
  }

  private void titleDrawStatus()
  {
    renderer_draw_side_boards();
    renderer_draw_score();
    title.draw();
  }

  private int screenShakeCnt;
  private float screenShakeIntense;

  public void setScreenShake(int cnt, float intense)
  {
    screenShakeCnt = cnt;
    screenShakeIntense = intense;
  }

  private void moveScreenShake()
  {
    if (screenShakeCnt > 0)
      screenShakeCnt--;
  }

  private void setEyepos()
  {
    float x = 0, y = 0;
    if (screenShakeCnt > 0)
    {
      x = rand.nextSignedFloat(screenShakeIntense * (screenShakeCnt + 10));
      y = rand.nextSignedFloat(screenShakeIntense * (screenShakeCnt + 10));
    }
    glTranslatef(x, y, -20);
  }

  public void draw()
  {
    screen.startRenderToTexture();
    glPushMatrix();
    setEyepos();
    switch (state)
    {
    case IN_GAME:
    case PAUSE:
    case GAMEOVER:
      inGameDrawLuminous();
      break;
    default:
    }
    glPopMatrix();
    screen.endRenderToTexture();

    screen.clear();
    glPushMatrix();
    setEyepos();
    switch (state)
    {
    case IN_GAME:
    case PAUSE:
      inGameDraw();
      break;
    case TITLE:
      titleDraw();
      break;
    case GAMEOVER:
      gameoverDraw();
      break;
    default:
    }
    glPopMatrix();

    screen.drawLuminous();

    screen.viewOrthoFixed();
    switch (state)
    {
    case IN_GAME:
      inGameDrawStatus();
      break;
    case TITLE:
      titleDrawStatus();
      break;
    case GAMEOVER:
      renderer_draw_gameover_status(stageManager.parsec, cnt);
      break;
    case PAUSE:
      renderer_draw_pause_status(stageManager.parsec, pauseCnt);
      break;
    default:
    }
    screen.viewPerspective();
  }
}
