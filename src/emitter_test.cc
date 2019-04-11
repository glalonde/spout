#include "src/emitter.h"
#include "base/googletest.h"

GTEST_TEST(EmitterTest, Smoke) {
  Emitter e(1.0, 10.0, 50.0, 20.0, 200);

  const double dt = .1;
  const SO2d start_angle(0.0);
  const SO2d end_angle(.2);
  const Vector2d start_pos(1.0, 2.0);
  const Vector2d end_pos(2.0, 3.0);
  const Vector2d start_vel(.1, .2);
  const Vector2d end_vel(.2, .3);
  e.EmitOverTime(dt, start_angle, end_angle, start_pos, end_pos, start_vel,
                 end_vel);
}

GTEST_MAIN();
