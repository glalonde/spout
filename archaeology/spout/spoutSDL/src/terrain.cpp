#include "terrain.h"
#include "collision.h"
#include <math.h>
#include <string.h>
#include <assert.h>
#include <iostream>

Terrain::Terrain() : TypeLayer(COLORS::GREY, COLORS::ORANGERED) {
  Reset();
}

void Terrain::Reset() {
  TypeLayer::Reset();
  this->bottom_level = 1;
  MakeLevel(bot_buff, bottom_level);
  MakeLevel(top_buff, bottom_level + 1);
}

void Terrain::SwapBuffers() {
  MakeLevel(bot_buff, ++bottom_level + 1);
  TypeLayer::SwapBuffers();
}

void Terrain::UnSwapBuffers() {
  MakeLevel(top_buff, --bottom_level);
  TypeLayer::UnSwapBuffers();
}

void Terrain::PrintOcc() {
  printf("----------------- %d\n", top_height);
  for (int i = top_height - 1; i >= bottom_height; i--) {
    for (int j = 0; j < GRID_WIDTH; j++) {
      // block_t block = GetBlock(j, GetBlockY(i));
      if (IsFull(j, i)) {
        printf("#");
      } else {
        printf(" ");
      }
    }
    printf("\n");
  }
  printf("----------------- %d\n", bottom_height);
  top_buff->PrintOcc();
  bot_buff->PrintOcc();
}

void Terrain::MakeLevel(TileBuffer* level_buffer, int level_num) {
  level_buffer->SetID(level_num);
  #ifdef SPEEDTEST
  MakeSpeedTestLevel(level_buffer, level_num);
  #else
  std::srand(level_num);
	if (level_num > 1) {
    GenerateRectLevel(level_buffer, ceil(LEVEL_WIDTH / level_num) / 2, (int)(LEVEL_HEIGHT * sqrt(level_num)));
	} else {
	  GenerateRectLevel(level_buffer, LEVEL_WIDTH / 2, (int)(LEVEL_HEIGHT * sqrt(level_num)));
    level_buffer->FillRect(0, 0, LEVEL_WIDTH, FIRST_LEVEL_EMPTY_HEIGHT, TERRAIN_TYPES::EMPTY);
	}
  //MarkEdges(level_buffer);
  #endif
}

void Terrain::MakeWebbingLevel(TileBuffer* level_buffer, int level_num) {
  std::srand(level_num);
  GenerateWebbingLevel(level_buffer, level_num);

	if (level_num <= 1) {
    level_buffer->FillRect(0, 0, LEVEL_WIDTH, FIRST_LEVEL_EMPTY_HEIGHT, TERRAIN_TYPES::EMPTY);
	}
}

void Terrain::MakeSpeedTestLevel(TileBuffer* level_buffer, int level_num) {
  level_buffer->FillRect(0, 0, LEVEL_WIDTH, LEVEL_HEIGHT, TERRAIN_TYPES::FULL);
}

void Terrain::GenerateWebbingLevel(TileBuffer* level_buffer, int n_lines) {
  level_buffer->CoolRect(0, 0, level_buffer->width, level_buffer->height, n_lines, TERRAIN_TYPES::FULL);
}

void Terrain::GenerateRectLevel(TileBuffer* level_buffer, int max_dimension, int num_vacancies) {
  level_buffer->SetAll(TERRAIN_TYPES::FULL);
  
  max_dimension = (max_dimension < 1)?1:max_dimension;
  
  for (int i = 0; i < num_vacancies; i++) {
    int width = std::rand()%max_dimension;
    int left = std::rand()%(LEVEL_WIDTH - width + 1);
    int height = std::rand()%max_dimension;
    int bot = std::rand()%(LEVEL_HEIGHT - height + 1);
    level_buffer->FillRect(left, bot, width, height, TERRAIN_TYPES::EMPTY);
  }
}
