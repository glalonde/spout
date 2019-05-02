#include <stdio.h>
#include <stdlib.h>
#include <time.h>   
#include <assert.h>


#define LEVEL_HEIGHT 1000
#define LEVEL_WIDTH 500

static const int width = LEVEL_WIDTH;
static const int height = LEVEL_HEIGHT;

#include "packed_grid.h"
#include "unpacked_grid.h"

void TestCorrect() {
  const int rounds = 10000;

  UnpackedGrid* ugrid = new UnpackedGrid();
  PackedGrid32* grid = new PackedGrid32();

  int x;
  int y;

  for (int i = 0; i < rounds; i ++) {
    x = rand()%width;
    y = rand()%height;
    ugrid->SetCell(x, y);
    grid->SetCell(x, y);
  }

  for (int x = 0; x < width; x++) {
    for (int y = 0; y < height; y++) {
      assert(grid->GetCell(x, y) == ugrid->GetCell(x, y));
    }
  }

  delete grid;
  delete ugrid;
}

void TestRandomRead() {
  const int rounds = 1000000;

  UnpackedGrid* ugrid = new UnpackedGrid();
  PackedGrid32* grid = new PackedGrid32();

  int start, end;
  int packed_time, unpacked_time;

  int xs[rounds], ys[rounds];
  int count = 0;

  srand(time(NULL));
  for (int i = 0; i < rounds; i ++) {
    xs[i] = rand()%width;
    ys[i] = rand()%height;
  }
  start = clock();
  for (int i = 0; i < rounds; i ++) {
    if (grid->GetCell(xs[i], ys[i])) {
      count++;
    }
  }
  end = clock();
  packed_time = end - start;

  start = clock();
  for (int i = 0; i < rounds; i ++) {
    if (ugrid->GetCell(xs[i], ys[i])) {
      count++;
    }
  }
  end = clock();
  unpacked_time = end - start;

  printf("packed time: %d\n", packed_time);
  printf("unpacked time: %d\n", unpacked_time);
  printf("count: %d\n", count);

  delete grid;
  delete ugrid;
}

void TestSequentialZero() {


  UnpackedGrid* ugrid = new UnpackedGrid();
  for (int x = 0; x < width; x++) {
    for (int y = 0; y < height; y++) {
      assert(ugrid->GetCell(x, y) == 0);
    }
  }
  PackedGrid32* grid = new PackedGrid32();

  int start, end;
  int packed_time, unpacked_time;

  bool empty1, empty2;

  start = clock();
  empty1 = grid->IsEmpty();
  end = clock();
  packed_time = end - start;

  start = clock();
  empty2 = ugrid->IsEmpty();
  end = clock();
  unpacked_time = end - start;

  printf("packed time: %d\n", packed_time);
  printf("unpacked time: %d\n", unpacked_time);
  printf("empty1: %d, empty2: %d\n", empty1, empty2);

  delete grid;
  delete ugrid;
}

void TestRect() {
  PackedGrid32* grid = new PackedGrid32();

  grid->FillRect(0,0, 4, 8);
  grid->FillRect(1,0, 4, 8);
  grid->FillRect(0,1, 4, 8);
  grid->FillRect(0,0, 3, 8);
  grid->FillRect(0,0, 4, 7);
  delete grid;
}



void TestDoBlock() {
  PackedGrid32* grid = new PackedGrid32();

  int x = 10;
  int y = 10;
  BoolIntVec res;

  res = grid->DoBlock(x/grid->PackedGrid32::BLOCK_WIDTH, y/grid->PackedGrid32::BLOCK_HEIGHT, x, y, 15, 16);
  printf("Collision: %d, x: %d, y: %d\n", res.test, res.vec.x, res.vec.y);

  res = grid->DoBlock(x/grid->PackedGrid32::BLOCK_WIDTH, y/grid->PackedGrid32::BLOCK_HEIGHT, x, y, 7, 16);
  printf("Collision: %d, x: %d, y: %d\n", res.test, res.vec.x, res.vec.y);

  res = grid->DoBlock(x/grid->PackedGrid32::BLOCK_WIDTH, y/grid->PackedGrid32::BLOCK_HEIGHT, x, y, 7, 15);
  printf("Collision: %d, x: %d, y: %d\n", res.test, res.vec.x, res.vec.y);


  grid->FillRect(8, 8, 1, 6);
  grid->FillRect(9, 14, 3, 1);

  printf("Now with stuff\n");

  // Down the middle
  res = grid->DoBlock(x/grid->PackedGrid32::BLOCK_WIDTH, y/grid->PackedGrid32::BLOCK_HEIGHT, x, y, 7, 16);
  printf("Collision: %d, x: %d, y: %d\n", res.test, res.vec.x, res.vec.y);
  assert(!res.test);
  assert(res.vec.x == 7 && res.vec.y == 16);

  res = grid->DoBlock(x/grid->PackedGrid32::BLOCK_WIDTH, y/grid->PackedGrid32::BLOCK_HEIGHT, x, y, 7, 16);
  assert(!res.test);

  res = grid->DoBlock(x/grid->PackedGrid32::BLOCK_WIDTH, y/grid->PackedGrid32::BLOCK_HEIGHT, x, y, 7, 15);
  printf("Collision: %d, x: %d, y: %d\n", res.test, res.vec.x, res.vec.y);

  grid->FillRect(8, 8, 1, 8);
  grid->FillRect(11, 8, 1, 8);
  grid->FillRect(8, 8, 3, 1);
  grid->FillRect(8, 15, 3, 1);

  res = grid->DoBlock(x/grid->PackedGrid32::BLOCK_WIDTH, y/grid->PackedGrid32::BLOCK_HEIGHT, x, y, 0, 1);
  printf("Collision: %d, x: %d, y: %d\n", res.test, res.vec.x, res.vec.y);
  
  delete grid;
}

void TestCornerCase() {
  PackedGrid32* grid = new PackedGrid32();

  int x = 8;
  int y = 8;
  BoolIntVec res;

  res = grid->DoBlock(x/grid->PackedGrid32::BLOCK_WIDTH, y/grid->PackedGrid32::BLOCK_HEIGHT, x, y, 20, 20);
  printf("Collision: %d, x: %d, y: %d\n", res.test, res.vec.x, res.vec.y);
  assert(!res.test);
  assert(res.vec.x == 12 && res.vec.y == 12);

  res = grid->DoBlock(x/grid->PackedGrid32::BLOCK_WIDTH, y/grid->PackedGrid32::BLOCK_HEIGHT, x, y, 20, 19);
  printf("Collision: %d, x: %d, y: %d\n", res.test, res.vec.x, res.vec.y);
  assert(!res.test);
  assert(res.vec.x == 12 && res.vec.y == 11);

  grid->SetCell(8, 9);
  grid->SetCell(9, 8);
  grid->SetCell(9, 10);
  grid->SetCell(10, 9);
  grid->SetCell(11, 10);
  grid->SetCell(10, 11);

  // Down the middle
  res = grid->DoBlock(x/grid->PackedGrid32::BLOCK_WIDTH, y/grid->PackedGrid32::BLOCK_HEIGHT, x, y, 20, 20);
  printf("Collision: %d, x: %d, y: %d\n", res.test, res.vec.x, res.vec.y);
  assert(res.test);
  assert(res.vec.x == 9 && res.vec.y == 8);

  delete grid;
}

void TestCheckPair() {
  PackedGrid32* grid = new PackedGrid32();

  IntVec p1 = IntVec{200, 11};
  IntVec p2 = IntVec{90, 15};

  grid->FillRect(91, 2, 1, 20);



  BoolIntVec res = grid->CheckPair(p1.x, p1.y, p2.x, p2.y);
  printf("Collision: %d, x: %d, y: %d\n", res.test, res.vec.x, res.vec.y);

  delete grid;
}


void TestBlockEdge() {
  IntVec res;
  //printf("X: %d, Y: %d\n", res.x, res.y);
  res = PackedGrid32::GetBlockEdge(3, 1, -2, -3);
  assert(res.x == 2);
  assert(res.y == -1);

  res = PackedGrid32::GetBlockEdge(3, 7, -1, 0);
  assert(res.x == -1);
  assert(res.y == 7);

  res = PackedGrid32::GetBlockEdge(3, 7, 0, -1);
  assert(res.x == 3);
  assert(res.y == -1);

  res = PackedGrid32::GetBlockEdge(2, 7, -(PackedGrid32::BLOCK_WIDTH), -(PackedGrid32::BLOCK_HEIGHT));
  assert(res.x == -1);

  res = PackedGrid32::GetBlockEdge(3, 7, -50*(PackedGrid32::BLOCK_WIDTH), -49*(PackedGrid32::BLOCK_HEIGHT));
  assert(res.y == 0);
  assert(res.x == -1);

  res = PackedGrid32::GetBlockEdge(3, 7, -49*(PackedGrid32::BLOCK_WIDTH), -50*(PackedGrid32::BLOCK_HEIGHT));
  assert(res.y == -1);
  assert(res.x == 0);

  res = PackedGrid32::GetBlockEdge(1, 1, -(PackedGrid32::BLOCK_WIDTH), -(PackedGrid32::BLOCK_HEIGHT));
  assert(res.y == -1);
  assert(res.x == 0);

  res = PackedGrid32::GetBlockEdge(1, 1, -(PackedGrid32::BLOCK_WIDTH), -(PackedGrid32::BLOCK_HEIGHT));
  assert(res.y == -1);
  assert(res.x == 0);

  res = PackedGrid32::GetBlockEdge(3, 0, -50*(PackedGrid32::BLOCK_WIDTH), 49*(PackedGrid32::BLOCK_HEIGHT));
  assert(res.y == 7);
  assert(res.x == -1);

  res = PackedGrid32::GetBlockEdge(3, 0, -49*(PackedGrid32::BLOCK_WIDTH), 50*(PackedGrid32::BLOCK_HEIGHT));
  assert(res.y == 8);
  assert(res.x == 0);

  res = PackedGrid32::GetBlockEdge(1, 4, -50*(PackedGrid32::BLOCK_WIDTH), 49*(PackedGrid32::BLOCK_HEIGHT));
  assert(res.y == 7);
  assert(res.x == -1);

  res = PackedGrid32::GetBlockEdge(1, 4, -49*(PackedGrid32::BLOCK_WIDTH), 50*(PackedGrid32::BLOCK_HEIGHT));
  assert(res.y == 8);
  assert(res.x == 0);

  res = PackedGrid32::GetBlockEdge(0, 7, 50*(PackedGrid32::BLOCK_WIDTH), -49*(PackedGrid32::BLOCK_HEIGHT));
  assert(res.y == 0);
  assert(res.x == 4);

  res = PackedGrid32::GetBlockEdge(0, 7, 49*(PackedGrid32::BLOCK_WIDTH), -50*(PackedGrid32::BLOCK_HEIGHT));
  assert(res.y == -1);
  assert(res.x == 3);


  res = PackedGrid32::GetBlockEdge(0, 1, PackedGrid32::BLOCK_WIDTH, PackedGrid32::BLOCK_HEIGHT);
  assert(res.y == PackedGrid32::BLOCK_HEIGHT);

  res = PackedGrid32::GetBlockEdge(1, 0, 2*PackedGrid32::BLOCK_WIDTH, 2*PackedGrid32::BLOCK_HEIGHT);
  assert(res.x == PackedGrid32::BLOCK_WIDTH);

  res = PackedGrid32::GetBlockEdge(0, 1, 2*PackedGrid32::BLOCK_WIDTH, 2*PackedGrid32::BLOCK_HEIGHT);
  assert(res.y == PackedGrid32::BLOCK_HEIGHT);

  res = PackedGrid32::GetBlockEdge(0, 0, -50, -49);
  assert(res.y == 0);
  assert(res.x == -1);

  res = PackedGrid32::GetBlockEdge(0, 0, -49, -50);
  assert(res.y == -1);
  assert(res.x == 0);

  // Corner cases
  res = PackedGrid32::GetBlockEdge(2, 2, -3, 6);
  assert(res.y == 8);
  assert(res.x == -1);

  // Bot left
  res = PackedGrid32::GetBlockEdge(0, 0, -1, -1);
  assert(res.y == -1);
  assert(res.x == -1);

  // Bot right
  res = PackedGrid32::GetBlockEdge(3, 0, 1, -1);
  assert(res.y == -1);
  assert(res.x == 4);

  // Top left
  res = PackedGrid32::GetBlockEdge(0, 7, -1, 1);
  assert(res.y == 8);
  assert(res.x == -1);

  // Top right
  res = PackedGrid32::GetBlockEdge(3, 7, 1, 1);
  assert(res.y == 8);
  assert(res.x == 4);

  // MORE CORNER CASES
  int big = 350;

  // Bot left
  res = PackedGrid32::GetBlockEdge(0, 0, -1*big, -1*big);
  assert(res.y == -1);
  assert(res.x == -1);

  // Bot right
  res = PackedGrid32::GetBlockEdge(3, 0, 1*big, -1*big);
  assert(res.y == -1);
  assert(res.x == 4);

  // Top left
  res = PackedGrid32::GetBlockEdge(0, 7, -1*big, 1*big);
  assert(res.y == 8);
  assert(res.x == -1);

  // Top right
  res = PackedGrid32::GetBlockEdge(3, 7, 1*big, 1*big);
  assert(res.y == 8);
  assert(res.x == 4);

  // Bot left
  res = PackedGrid32::GetBlockEdge(0, 0, -1*big - 1, -1*big);
  assert(res.y == 0);
  assert(res.x == -1);

  res = PackedGrid32::GetBlockEdge(0, 0, -1*big, -1*big - 1);
  assert(res.y == -1);
  assert(res.x == 0);

  // Bot right
  res = PackedGrid32::GetBlockEdge(3, 0, 1*big + 1, -1*big);
  assert(res.y == 0);
  assert(res.x == 4);

  res = PackedGrid32::GetBlockEdge(3, 0, 1*big, -1*big - 1);
  assert(res.y == -1);
  assert(res.x == 3);

  // Top left
  res = PackedGrid32::GetBlockEdge(0, 7, -1*big - 1, 1*big);
  assert(res.y == 7);
  assert(res.x == -1);

  res = PackedGrid32::GetBlockEdge(0, 7, -1*big, 1*big + 1);
  assert(res.y == 8);
  assert(res.x == 0);

  // Top right
  res = PackedGrid32::GetBlockEdge(3, 7, 1*big + 1, 1*big);
  assert(res.y == 7);
  assert(res.x == 4);

  res = PackedGrid32::GetBlockEdge(3, 7, 1*big, 1*big + 1);
  assert(res.y == 8);
  assert(res.x == 3);

  // Edge case
  res = PackedGrid32::GetBlockEdge(2, 0, 1, 1);
  assert(res.y == 2);
  assert(res.x == 4);
}


int main( int argc, char* args[] ) {
  /*
  TestCorrect(); 
  TestRandomRead();
  TestSequentialZero();
  TestBlockBresenham();
  */
 // TestBlockEdge();

  //TestDoBlock();
  //TestCornerCase();
  TestCheckPair();

  return 0;
}
