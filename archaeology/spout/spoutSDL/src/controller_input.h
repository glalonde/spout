#ifndef CONTROLLER_INPUT_H_
#define CONTROLLER_INPUT_H_

/* Struct representing the state of the abstract controller
 at an instance in time */
struct ControllerInput {
  double time;
  bool up = false;
  bool down = false;
  bool left = false;
  bool right = false;
  bool reset = false;
  bool pause = false;
}; typedef struct ControllerInput ControllerInput;

#endif