alias BulletMLParserTinyXML = int;
alias BulletMLParser = int;
alias BulletMLState = int;
alias BulletMLRunner = int;
alias BulletMLRunnerD = int;

private alias BML_fp_d    = extern(C) double function(int*);
private alias BML_fp_i    = extern(C) int    function(int*);
private alias BML_fp_v    = extern(C) void   function(int*);
private alias BML_fp_vd   = extern(C) void   function(int*, double);
private alias BML_fp_vdd  = extern(C) void   function(int*, double, double);
private alias BML_fp_vsdd = extern(C) void   function(int*, BulletMLState*, double, double);

extern (C) {
int* BulletMLParserTinyXML_new(char*);
void BulletMLParserTinyXML_parse(int*);
void BulletMLParserTinyXML_delete(int*);
int* BulletMLRunner_new_parser(BulletMLParser*);
int* BulletMLRunner_new_state(BulletMLState*);
void BulletMLRunner_delete(int*);
void BulletMLRunner_run(int*);
bool BulletMLRunner_isEnd(int*);
void BulletMLRunner_set_getBulletDirection(int*, BML_fp_d);
void BulletMLRunner_set_getAimDirection(int*, BML_fp_d);
void BulletMLRunner_set_getBulletSpeed(int*, BML_fp_d);
void BulletMLRunner_set_getDefaultSpeed(int*, BML_fp_d);
void BulletMLRunner_set_getRank(int*, BML_fp_d);
void BulletMLRunner_set_createSimpleBullet(int*, BML_fp_vdd);
void BulletMLRunner_set_createBullet(int*, BML_fp_vsdd);
void BulletMLRunner_set_getTurn(int*, BML_fp_i);
void BulletMLRunner_set_doVanish(int*, BML_fp_v);
void BulletMLRunner_set_doChangeDirection(int*, BML_fp_vd);
void BulletMLRunner_set_doChangeSpeed(int*, BML_fp_vd);
void BulletMLRunner_set_doAccelX(int*, BML_fp_vd);
void BulletMLRunner_set_doAccelY(int*, BML_fp_vd);
void BulletMLRunner_set_getBulletSpeedX(int*, BML_fp_d);
void BulletMLRunner_set_getBulletSpeedY(int*, BML_fp_d);
void BulletMLRunner_set_getRand(int*, BML_fp_d);
}
