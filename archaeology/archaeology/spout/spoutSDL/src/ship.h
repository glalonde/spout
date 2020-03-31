#ifndef SHIP_H_
#define SHIP_H_
#include "mobile_object.h"
#include "controller_input.h"
#include "emitter.h"
#include "types.h"


class Ship : public MobileObject {
  static const int ROTATION_FREQ = 5.0;
  public:
    Ship();
    Ship(Vec pos, double direction);
  
    virtual void Reset(Vec pos, double direction);
  
    void Draw(Screen<counter_t>* screen);
  
    void HandleInput(ControllerInput* input);
    void Rotate(double radians);
    void Accelerate(double pixels_s_s);


    Emitter emitter = Emitter(SHIP_EMISSION_SPREAD, SHIP_EMISSION_MIN_SPEED, SHIP_EMISSION_MAX_SPEED, SHIP_EMISSION_RATE, PARTICLE_LIFE);

    double direction;
    double emission;
  
  protected:
    Vec exhaust_point;
    Vec tail_mid;
    Vec tail_min;
    Vec tail_max;
};


#endif
