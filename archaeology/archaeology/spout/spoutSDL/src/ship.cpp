#include "ship.h"
#include "color_utils.h"
#include "palette.h"

#define _USE_MATH_DEFINES
#include <math.h>
#include <iostream>


Ship::Ship() : direction(0), emission(0) {
  // Initialize the rotation-cache values
  Rotate(0);
}

Ship::Ship(Vec pos, double direction) : MobileObject(true, pos, pos, Vec_make(0,0)), direction(direction), emission(0) {
  Rotate(0);
}

void Ship::Reset(Vec pos, double direction) {
  this->direction = direction;
  this->emission = 0;
  Rotate(0);
  MobileObject::Reset(true, pos, pos, Vec_make(0,0));
  emitter.Reset();
}

void Ship::HandleInput(ControllerInput* input) {
  Rotate((input->left - input->right)*input->time*SHIP_ROTATION_SPEED);
  if (input->up) {
    Accelerate(input->time*SHIP_ACCELERATION);
    emitter.EmitOverTime(input->time, Vec_add(tail_mid, pos), velocity, direction + M_PI);
  }
}

void Ship::Rotate(double radians) {
  direction += radians;
  direction = fmod(direction, M_PI*2);
  if (direction < 0) direction += M_PI*2;
  exhaust_point = Vec_add(pos, Vec_multiply(Vec_unit(direction - M_PI), 4));
  tail_mid = Vec_multiply(Vec_unit(direction - M_PI), SHIP_TAIL_LENGTH/2);
  tail_min = Vec_multiply(Vec_unit(direction - M_PI - SHIP_EMISSION_SPREAD/2), SHIP_TAIL_LENGTH);
  tail_max = Vec_multiply(Vec_unit(direction - M_PI + SHIP_EMISSION_SPREAD/2), SHIP_TAIL_LENGTH);
}

void Ship::Accelerate(double pixels_s_s) {
  velocity = Vec_add(velocity, Vec_multiply(Vec_unit(direction), pixels_s_s));
}

static const counter_t TIP_COLOR = TerrainPalette::MAGENTA;
static const counter_t TAIL_COLOR = TerrainPalette::WHITE;
void Ship::Draw(Screen<counter_t>* screen) {
  screen->DrawLine((int)pos.x, (int)pos.y, (int)pos.x + (int)tail_min.x, (int)pos.y + (int)tail_min.y, TAIL_COLOR);
  screen->DrawLine((int)pos.x + tail_mid.x, (int)pos.y + (int)tail_mid.y, (int)pos.x + (int)tail_min.x, (int)pos.y + (int)tail_min.y, TAIL_COLOR);

  screen->DrawLine((int)pos.x, (int)pos.y, (int)pos.x + (int)tail_max.x, (int)pos.y + (int)tail_max.y, TAIL_COLOR);
  screen->DrawLine((int)(pos.x + tail_mid.x), (int)pos.y + (int)tail_mid.y, (int)pos.x + (int)tail_max.x, (int)pos.y + (int)tail_max.y, TAIL_COLOR);

  screen->SetCell(pos.x, pos.y, TIP_COLOR);
}
