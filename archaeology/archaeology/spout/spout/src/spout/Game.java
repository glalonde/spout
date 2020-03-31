package spout;

import static spout.GlobalConstants.GRID_HEIGHT;
import static spout.GlobalConstants.GRID_WIDTH;

public class Game {
    public boolean isOver = false;

    // Game Environment Constants
    int[][] prevScreenBuffer = new int[GRID_WIDTH][GRID_HEIGHT];
    CellType[][] screenBuffer = new CellType[GRID_WIDTH][GRID_HEIGHT];
    boolean[][] deltaBuffer = new boolean[GRID_WIDTH][GRID_HEIGHT];

    public Game() {
	for (int x = 0; x < GRID_WIDTH; x++) {
	    for (int y = 0; y < GRID_HEIGHT; y++) {
		screenBuffer[x][y] = CellType.EMPTY;
		deltaBuffer[x][y] = true;
	    }
	}
    }

    MobileObjects mobileObjects = new MobileObjects();

    public void step(boolean keyExhaust, boolean keyTurnLeft,
	    boolean keyTurnRight) {

	if (keyExhaust) {
	    mobileObjects.emit();
	}
	// GAME LOGIC
	mobileObjects.updateGrains();

	// SHIP
	if (mobileObjects.updateShip(keyExhaust, keyTurnLeft, keyTurnRight)) {
	    isOver = true;
	}

	// GRAPHICS STUFF, HAPPENS AFTER ALL THE COMPUTATION-----------------
	// clone the old screenbuffer, so we can determine what is different and
	// only draw that.
	for (int x = 0; x < GRID_WIDTH; x++) {
	    for (int y = 0; y < GRID_HEIGHT; y++) {
		prevScreenBuffer[x][y] = screenBuffer[x][y].ordinal();
	    }
	}

	mobileObjects.setScreen(screenBuffer);
	generateDeltaBuffer();
    }

    // figure out if a particular spot has changed between frames
    // useful for long patches of the same color
    private void generateDeltaBuffer() {
	for (int x = 0; x < GRID_WIDTH; x++) {
	    for (int y = 0; y < GRID_HEIGHT; y++) {
		if (screenBuffer[x][y].ordinal() != prevScreenBuffer[x][y]) {
		    deltaBuffer[x][y] = true;
		} else {
		    deltaBuffer[x][y] = false;
		}
	    }
	}
    }

}
