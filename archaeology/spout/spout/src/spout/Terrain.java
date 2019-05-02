package spout;

import static spout.GlobalConstants.GRID_HEIGHT;
import static spout.GlobalConstants.GRID_WIDTH;
import static spout.GlobalConstants.LEVEL_HEIGHT;

import java.util.Random;

public class Terrain {
    private boolean[][] topLevelBuffer = new boolean[GRID_WIDTH][LEVEL_HEIGHT];
    private boolean[][] bottomLevelBuffer = new boolean[GRID_WIDTH][LEVEL_HEIGHT];

    public int currentLevel = 1;
    private Random rand = new Random();

    Terrain() {
	makeLevel(bottomLevelBuffer, currentLevel++);
	makeLevel(topLevelBuffer, currentLevel++);
    }

    // EXTERNAL CALLS
    public boolean isFull(int x, int y, int screenBottom) {
	if (x < 0 || x >= GRID_WIDTH || y < screenBottom) {
	    return true;
	} else if (y >= LEVEL_HEIGHT) {
	    if (topLevelBuffer[x][y % LEVEL_HEIGHT]) {
		remove(x, y);
		return true;
	    } else {
		return false;
	    }
	} else {
	    if (bottomLevelBuffer[x][y]) {
		remove(x, y);
		return true;
	    } else {
		return false;
	    }
	}
    }

    public void cycleBuffers() {
	boolean[][] tempLevelBuffer = bottomLevelBuffer;
	bottomLevelBuffer = topLevelBuffer;
	makeLevel(tempLevelBuffer, currentLevel++);
	topLevelBuffer = tempLevelBuffer;
    }

    public void remove(int x, int y) {
	if (y >= LEVEL_HEIGHT) {
	    topLevelBuffer[x][y % LEVEL_HEIGHT] = false;
	} else {
	    bottomLevelBuffer[x][y] = false;
	}
    }

    public void setScreen(int screenBottom, CellType[][] screenBuffer) {
	if (screenBottom < LEVEL_HEIGHT - GRID_HEIGHT) {
	    // we can read from one buffer
	    for (int x = 0; x < GRID_WIDTH; x++) {
		for (int y = 0; y < GRID_HEIGHT; y++) {
		    screenBuffer[x][y] = (bottomLevelBuffer[x][y + screenBottom]) ? CellType.GROUND
			    : CellType.EMPTY;
		}
	    }
	} else {
	    int botHeight = LEVEL_HEIGHT - screenBottom;
	    int topHeight = GRID_HEIGHT - botHeight;

	    for (int x = 0; x < GRID_WIDTH; x++) {
		for (int y = 0; y < botHeight; y++) {
		    screenBuffer[x][y] = (bottomLevelBuffer[x][y + screenBottom]) ? CellType.GROUND
			    : CellType.EMPTY;
		}
	    }

	    for (int x = 0; x < GRID_WIDTH; x++) {
		for (int y = 0; y < topHeight; y++) {
		    screenBuffer[x][botHeight + y] = (topLevelBuffer[x][y]) ? CellType.GROUND
			    : CellType.EMPTY;
		}
	    }
	}
    }

    // LOCAL CALLS

    private final int FIRST_LEVEL_EMPTY_HEIGHT = 2 * GRID_HEIGHT / 3;

    private void makeLevel(boolean[][] levelBuffer, int levelNum) {
	final int BUFFER_WIDTH = levelBuffer.length;
	final int BUFFER_HEIGHT = levelBuffer[0].length;
	if (levelNum > 1) {
	    generateLevel(levelBuffer,
		    (int) Math.ceil(BUFFER_WIDTH / levelNum) / 2,
		    (int) (BUFFER_HEIGHT * Math.sqrt(levelNum)));
	} else {
	    generateLevel(levelBuffer, BUFFER_WIDTH / 2,
		    (int) (BUFFER_HEIGHT * Math.sqrt(levelNum)));
	    for (int x = 0; x < BUFFER_WIDTH; x++) {
		for (int y = 0; y < FIRST_LEVEL_EMPTY_HEIGHT; y++) {
		    levelBuffer[x][y] = false;
		}
	    }
	}
    }

    private void generateLevel(boolean[][] levelBuffer, int maxDimension,
	    int numVacancies) {
	final int BUFFER_WIDTH = levelBuffer.length;
	final int BUFFER_HEIGHT = levelBuffer[0].length;
	for (int x = 0; x < BUFFER_WIDTH; x++) {
	    for (int y = 0; y < levelBuffer[x].length; y++) {
		levelBuffer[x][y] = true;
	    }
	}
	for (int i = 0; i < numVacancies; i++) {
	    int width = rand.nextInt(maxDimension);
	    int left = rand.nextInt(BUFFER_WIDTH - width);
	    int right = left + width + 1;

	    int height = rand.nextInt(maxDimension);
	    int bot = rand.nextInt(BUFFER_HEIGHT - height);
	    int top = bot + height + 1;

	    for (int x = left; x < right; x++) {
		for (int y = bot; y < top; y++) {
		    levelBuffer[x][y] = false;
		}
	    }
	}
    }
}
