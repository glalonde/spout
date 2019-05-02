#include "type_layer.h"
#include "color_utils.h"
#include <assert.h>

TypeLayer::TypeLayer () : bot_buff(&buff_1),
                          top_buff(&buff_2),
                          height(bot_buff->height + top_buff->height),
                          width(buff_1.width),
                          bottom_height(0),
                          middle_height(bottom_height + bot_buff->height),
                          top_height(middle_height + top_buff->height) {}

TypeLayer::TypeLayer (pixel_t start_color, pixel_t end_color) :
                      buff_1(end_color, start_color),
                      buff_2(end_color, start_color),
                      bot_buff(&buff_1),
                      top_buff(&buff_2),
                      height(bot_buff->height + top_buff->height),
                      width(buff_1.width),
                      bottom_height(0),
                      middle_height(bottom_height + bot_buff->height),
                      top_height(middle_height + top_buff->height) {}


void TypeLayer::Reset() {
  bot_buff->Reset();
  top_buff->Reset();
  bottom_height = 0;
  middle_height = bottom_height + bot_buff->height;
  top_height = bottom_height + bot_buff->height + top_buff->height;
}

void TypeLayer::SwapBuffers() {
  this->bottom_height += bot_buff->height;
  this->middle_height += bot_buff->height;
  this->top_height += bot_buff->height;
  std::swap(bot_buff, top_buff);
}

void TypeLayer::UnSwapBuffers() {
  this->bottom_height -= top_buff->height;
  this->middle_height -= top_buff->height;
  this->top_height -= top_buff->height;
  std::swap(bot_buff, top_buff);
}

bool TypeLayer::SyncHeight(int screen_bottom) {
  if (screen_bottom > this->middle_height) {
    SwapBuffers();
    return true;
  } else if (screen_bottom < bottom_height) {
    UnSwapBuffers();
    return true;
  }
  return false;
}

void TypeLayer::Draw(Screen<counter_t>* screen) {
  bot_buff->Draw(screen, this->bottom_height);
  top_buff->Draw(screen, this->middle_height);
}
