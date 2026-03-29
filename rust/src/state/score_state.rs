use crate::sound::*;

pub struct ScoreState {
    score: i32,
    bonus_score: i32,
    extend_score: i32,
    life: i32,
}

const MAX_BONUS_SCORE: i32 = 1000;
const BONUS_INCREMENT: i32 = 10;
const MAX_LIFE: i32 = 4;
const INITIAL_LIFE: i32 = 2;
const FIRST_EXTEND: i32 = 200000;
const EVERY_EXTEND: i32 = 500000;

impl ScoreState {
    pub const fn new() -> Self {
        ScoreState { bonus_score: BONUS_INCREMENT, life: INITIAL_LIFE, extend_score: 0, score: 0 }
    }

    pub fn reset_bonus_score(&mut self) {
        self.bonus_score = BONUS_INCREMENT;
    }

    pub fn increase_bonus_score(&mut self) {
        if self.bonus_score < MAX_BONUS_SCORE {
            self.bonus_score += BONUS_INCREMENT;
        }
    }

    pub fn get_bonus_score(&self) -> i32 {
        self.bonus_score
    }

    pub fn set_initial(&mut self) {
        self.score = 0;
        self.extend_score = FIRST_EXTEND;
        self.life = INITIAL_LIFE;
    }

    pub fn get_life(&self) -> i32 {
        self.life
    }

    pub fn decrease_life(&mut self) {
        self.life -= 1;
    }

    pub fn get_score(&self) -> i32 {
        self.score
    }

    pub fn increase_score(&mut self, sc: i32) {
        self.score += sc;

        if self.score > self.extend_score {
            if self.life < MAX_LIFE {
                sound_manager_play_se(5);
                self.life += 1;
            }
            if self.extend_score <= FIRST_EXTEND {
                self.extend_score = EVERY_EXTEND;
            } else {
                self.extend_score += EVERY_EXTEND;
            }
        }
    }

    pub fn bonus_collected(&mut self) {
        self.increase_score(self.get_bonus_score());
        self.increase_bonus_score();
        sound_manager_play_se(4);
    }
}
