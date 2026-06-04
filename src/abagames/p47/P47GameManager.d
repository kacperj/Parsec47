/*
 * $Id: P47GameManager.d,v 1.6 2004/01/01 11:26:42 kenta Exp $
 *
 * Copyright 2003 Kenta Cho. All rights reserved.
 */
module abagames.p47.P47GameManager;

private:
import std.math;
import bulletml;
import abagames.util.Rand;
import abagames.util.Vector;
import abagames.util.ActorPool;
import abagames.util.sdl.MainLoop;
import abagames.util.sdl.Pad;
import abagames.p47.P47PrefManager;
import abagames.p47.Ship;
import abagames.p47.Field;
import abagames.p47.Enemy;
import abagames.p47.EnemyType;
import abagames.p47.BulletActor;
import abagames.p47.BulletActorPool;
import abagames.p47.BarrageManager;
import abagames.p47.Lock;
import abagames.p47.StageManager;
import abagames.p47.Title;
import abagames.p47.SoundManager;
import abagames.p47.Renderer;
import abagames.p47.ShipMode;
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
  void letter_render_create_display_lists();

  void particles_draw();
  void particles_update();

  void fragments_draw();
  void fragments_update();

  void score_set_initial();
  void life_decrease();
  int life_get();
  int score_get();

  void shots_update();
  void shots_draw();
  void shots_clear();

  void rolls_clear();
  void rolls_draw();
  void rolls_update();

  void bonuses_init();
  void bonuses_set_speed_rate(float r);
  void bonuses_clear();
  void bonuses_move();
  void bonuses_draw();

  void screen_shake_set(int cnt, float intense);
  void screen_shake_update();
  void screen_shake_apply();

  void screen_clear();
  void screen_draw_luminous();
  void screen_view_ortho_fixed();
  void screen_view_perspective();

  void game_manager_draw_luminous(int state);

  void gl_push_matrix();
  void gl_pop_matrix();

  int prefs_get_start_parsec(int mode, int difficulty, int slot);
}

public class P47GameManager
{
public:
  int status;
  bool nowait = false;
  static bool noBonus = false;
  int difficulty, parsecSlot;

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

  public void setMainLoop(MainLoop mainLoop)
  {
    this.mainLoop = mainLoop;
  }

private:
  Pad pad;
  const int ENEMY_MAX = 32;
  Rand rand;
  Field field;
  Ship ship;
  ActorPool!Enemy enemies;
  BulletActorPool bullets;
  ActorPool!Lock locks;
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
    this.difficulty = prefs_get_selected_difficulty();
    this.parsecSlot = prefs_get_selected_parsec_slot();
    this.mode = prefs_get_selected_mode();

    pad = new Pad;
    rand = new Rand;
    field_create_ring_display_list();
    field = new Field;
    field_init(Box.createWithHalfExtents(11, 16));
    ship_create_display_lists();
    ship = new Ship;
    ship.init(pad, field, this);
    bullet_actor_create_display_lists();
    bullets = new BulletActorPool(512, () => new BulletActor(field, ship));
    letter_render_create_display_lists();
    Lock.init();
    locks = new ActorPool!Lock(4, () => new Lock());
    enemies = new ActorPool!Enemy(ENEMY_MAX, () => new Enemy(field, bullets, locks, ship, this));
    bonuses_init();
    barrageManager = new BarrageManager;
    barrageManager.loadBulletMLs();
    EnemyType.init(barrageManager);
    stageManager = new StageManager;
    stageManager.init(this, field);
    title = new Title;
    renderer_title_texture_init();
    interval = mainLoop.INTERVAL_BASE;
    cast(void) sound_manager_init();
  }

  public void start()
  {
    startTitle();
  }

  public void close()
  {
    barrageManager.unloadBulletMLs();
    renderer_title_texture_delete();
    sound_manager_close();
  }

  public void shipDestroyed()
  {
    clearBullets();
    life_decrease();
    if (life_get() < 0)
      startGameover();
  }

  public void addEnemy(Vector2 pos, float d, EnemyType type, int moveType, int moveTypeRandom)
  {
    Enemy en = enemies.getInstance();
    if (!en)
      return;

    BulletMLParser* moveParser = barrageManager.getMoveParser(moveType, moveTypeRandom);
    BulletMLRunner* moveRunner = BulletMLRunner_new_parser(moveParser);
    en.set(pos, d, type, moveRunner);
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

  public void addLock()
  {
    Lock lock = locks.getInstance();
    if (!lock)
      return;
    lock.set();
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

  public void startStagePreview()
  {
    StageSelection stageSelection = title.getStatus();
    startStage(stageSelection.difficulty, stageSelection.parsecSlot, stageSelection.mode);
  }

  public void startStage(int difficulty, int parsecSlot, int mode)
  {
    int startParsec = prefs_get_start_parsec(mode, difficulty, parsecSlot);

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
      bonuses_set_speed_rate(0.6);
      break;
    case NORMAL:
      stageManager.setRank(10, 8, startParsec, stageType);
      ship.setSpeedRate(0.9);
      bonuses_set_speed_rate(0.8);
      break;
    case HARD:
      stageManager.setRank(22, 12, startParsec, stageType);
      ship.setSpeedRate(1);
      bonuses_set_speed_rate(1);
      break;
    case EXTREME:
      stageManager.setRank(36, 16, startParsec, stageType);
      ship.setSpeedRate(1.2);
      bonuses_set_speed_rate(1.3);
      break;
    case QUIT:
      stageManager.setRank(0, 0, 0, 0);
      ship.setSpeedRate(1);
      bonuses_set_speed_rate(1);
      break;
    default:
      break;
    }
  }

  private void initShipState()
  {
    score_set_initial();
    ship.start(mode);
  }

  private void startInGame()
  {
    state = IN_GAME;
    SoundManager.isInGame = (state == IN_GAME);
    initShipState();
    startStage(difficulty, parsecSlot, mode);
  }

  private void startTitle()
  {
    state = TITLE;
    title_start(difficulty, parsecSlot, mode);
    field.setColor(mode);
    initShipState();
    bullets.clear();
    bonuses_clear();
    ship.cnt = 0;
    startStagePreview();
    cnt = 0;
    sound_manager_stop_music();
  }

  private void startGameover()
  {
    state = GAMEOVER;
    bonuses_clear();
    shots_clear();
    rolls_clear();
    locks.clear();
    setScreenShake(0, 0);
    interval = mainLoop.INTERVAL_BASE;
    mainLoop.interval = mainLoop.INTERVAL_BASE;
    cnt = 0;
    if (score_get() > prefs_get_hi_score(mode, difficulty, parsecSlot))
      prefs_set_hi_score(mode, difficulty, parsecSlot, score_get());
    if (stageManager.parsec > prefs_get_reached_parsec(mode, difficulty))
      prefs_set_reached_parsec(mode, difficulty, stageManager.parsec);
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

  private bool pPrsd = true;

  private void inGameMove()
  {
    stageManager.move();
    field.move();
    ship.move();
    bonuses_move();
    shots_update();
    enemies.move();
    if (mode == ShipMode.ROLL)
      rolls_update();
    else
      locks.move();
    BulletActor.resetTotalBulletsSpeed();
    bullets.move();
    particles_update();
    fragments_update();
    screen_shake_update();
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
    title_move();
    if (title_should_change_stage() != 0) {
      startStagePreview();
    }

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
          StageSelection stageSelection = title.getStatus();
          this.difficulty = stageSelection.difficulty;
          this.parsecSlot = stageSelection.parsecSlot;
          this.mode = stageSelection.mode;

          if (difficulty >= P47PrefManager.DIFFICULTY_NUM)
            mainLoop.breakLoop();
          else {
            prefs_set_selected_difficulty(this.difficulty);
            prefs_set_selected_parsec_slot(this.parsecSlot);
            prefs_set_selected_mode(this.mode);
            startInGame();
          }

          return;
        }
      }
      else if (pad.isButton2())
      {
        if (!btnPrsd)
        {
          title_change_mode();
          startStagePreview();
          field.setColor(mode);
          btnPrsd = true;
        }
      }
      else
      {
        btnPrsd = false;
      }
    }
    stageManager.move();
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
    if (pad_is_key_pressed(27) != 0)
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
    if (!noBonus)
      bonuses_draw();


    particles_draw();
    fragments_draw();
    ship.draw();
    shots_draw();
    
    if (mode == ShipMode.ROLL)
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
    particles_draw();
    fragments_draw();
    enemies.draw();
    bullets.draw();
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
    title_draw();
  }

  public void setScreenShake(int cnt, float intense)
  {
    screen_shake_set(cnt, intense);
  }

  public void draw()
  {
    game_manager_draw_luminous(state);

    screen_clear();
    gl_push_matrix();
    screen_shake_apply();
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
    gl_pop_matrix();

    screen_draw_luminous();

    screen_view_ortho_fixed();
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
    screen_view_perspective();
  }
}
