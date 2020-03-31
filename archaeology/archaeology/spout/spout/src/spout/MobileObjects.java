package spout;

import static spout.GlobalConstants.CELL_SIZE;
import static spout.GlobalConstants.COLLISION_ELASTICITY;
import static spout.GlobalConstants.EMISSION_ANGLE;
import static spout.GlobalConstants.EMISSION_RATE;
import static spout.GlobalConstants.EMISSION_VELOCITY;
import static spout.GlobalConstants.EXHAUST_ACCEL;
import static spout.GlobalConstants.GRAIN_LIFE;
import static spout.GlobalConstants.GRAVITY;
import static spout.GlobalConstants.GRID_HEIGHT;
import static spout.GlobalConstants.GRID_WIDTH;
import static spout.GlobalConstants.LEVEL_HEIGHT;
import static spout.GlobalConstants.MAX_GRAIN_VELOCITY;
import static spout.GlobalConstants.MAX_SHIP_VELOCITY;
import static spout.GlobalConstants.NUM_GRAINS;
import static spout.GlobalConstants.SCROLL_HEIGHT;
import static spout.GlobalConstants.TURN_VELOCITY;

//import static spout.GlobalConstants.*;
public class MobileObjects {

    /*
     * MOBILE OBJECT DATA Ship is at position 0, grains are 1 through NUM_GRAINS
     */
    // int[][] grainCounts = new int[GRID_WIDTH][GRID_HEIGHT];

    int[] x = new int[NUM_GRAINS + 1];
    int[] y = new int[NUM_GRAINS + 1];
    int[] gridX = new int[NUM_GRAINS + 1];
    int[] gridY = new int[NUM_GRAINS + 1];

    // Previous x and y for use with collision detection and response
    int[] prevX = new int[NUM_GRAINS + 1];
    int[] prevY = new int[NUM_GRAINS + 1];
    int[] prevGridX = new int[NUM_GRAINS + 1];
    int[] prevGridY = new int[NUM_GRAINS + 1];

    // Velocities
    // positive in the rightward direction
    int[] dx = new int[NUM_GRAINS + 1];
    // positive in the upward direction
    int[] dy = new int[NUM_GRAINS + 1];

    boolean[] isActive = new boolean[NUM_GRAINS + 1];
    int[] TTL = new int[NUM_GRAINS + 1];

    // Ship direction
    double direction;
    Terrain terrain = new Terrain();

    int score;

    MobileObjects() {
	score = 0;
	// Initialize grains
	for (int i = 1; i <= NUM_GRAINS; i++) {
	    x[i] = 0;
	    y[i] = 0;
	    setGridLoc(i);
	    prevX[i] = x[0];
	    prevY[i] = y[0];
	    prevGridX[i] = gridX[i];
	    prevGridY[i] = gridY[i];
	    dx[i] = 0;
	    dy[i] = 0;
	    TTL[i] = 0;
	    isActive[i] = false;
	}

	// Initialize ship
	x[0] = (GRID_WIDTH / 2) * CELL_SIZE + CELL_SIZE / 2;
	y[0] = (GRID_HEIGHT / 2) * CELL_SIZE + CELL_SIZE / 2;
	setGridLoc(0);
	prevX[0] = x[0];
	prevY[0] = y[0];
	prevGridX[0] = gridX[0];
	prevGridY[0] = gridY[0];
	dx[0] = 0;
	dy[0] = 0;
	TTL[0] = 0;
	isActive[0] = true;
	direction = Math.PI / 2;
    }

    public void updateGrains() {
	for (int i = 1; i <= NUM_GRAINS; i++) {
	    if (isActive[i]) {

		// Apply gravity
		dy[i] += GRAVITY;
		updateAbsoluteVelocity(i, MAX_GRAIN_VELOCITY);

		// Update position data
		prevX[i] = x[i];
		prevY[i] = y[i];
		x[i] += dx[i];
		y[i] += dy[i];
		updateGridLoc(i);

		getCollision(i);

		// Update life
		if (--TTL[i] < 0)
		    isActive[i] = false;

	    }
	}
    }

    private int currentGrain = 1;

    public void emit() {
	for (int i = 0; i < EMISSION_RATE; i++) {
	    emit(++currentGrain);
	    currentGrain %= NUM_GRAINS;
	}
    }

    public boolean updateShip(boolean exhaust, boolean turnLeft,
	    boolean turnRight) {

	// Apply gravity
	dy[0] += GRAVITY;

	// Turn appropriately
	if (turnLeft) {
	    if (!turnRight) {
		direction += TURN_VELOCITY;
	    }
	} else if (turnRight) {
	    if (!turnLeft) {
		direction -= TURN_VELOCITY;
	    }
	}

	// If exhaust, initialize new grains
	if (exhaust) {
	    updateVelocity(0, direction, EXHAUST_ACCEL);
	}

	updateAbsoluteVelocity(0, MAX_SHIP_VELOCITY);
	// Update position data
	prevX[0] = x[0];
	prevY[0] = y[0];
	x[0] += dx[0];
	y[0] += dy[0];
	updateGridLoc(0);

	// Scroll screen as required
	int scrollDisplacement = toScreenY(gridY[0]) - SCROLL_HEIGHT;
	// System.out.println(scrollDisplacement);
	if (scrollDisplacement > 0) {
	    screenBottom += scrollDisplacement;
	    score += scrollDisplacement;
	    // If we have gotten sufficiently far onto a new level that we can
	    // no longer see the old one, swap the terrain buffers
	    // System.out.println(screenBottom);
	    if (screenBottom > LEVEL_HEIGHT) {
		modHeightData(LEVEL_HEIGHT);
		terrain.cycleBuffers();
		screenBottom = scrollDisplacement;
	    }
	}
	// System.out.println("Ship x: " + x[0] + " y: " + y[0] + " gridX: "
	// + gridX[0] + " gridY: " + gridY[0]);
	// System.out.println(terrain.isFull(gridX[0], gridY[0]));
	return getCollision(0);
    }

    public void updateVelocity(int i, double angle, int ddv) {
	dx[i] += ddv * Math.cos(angle);
	dy[i] += ddv * Math.sin(angle);
    }

    private void setGridLoc(int i) {
	gridX[i] = x[i] - x[i] % CELL_SIZE;
	gridY[i] = y[i] - y[i] % CELL_SIZE;
    }

    protected void updateGridLoc(int i) {
	prevGridX[i] = gridX[i];
	prevGridY[i] = gridY[i];
	setGridLoc(i);
    }

    public void updateAbsoluteVelocity(int i, int maxVelocity) {
	double dv = (dy[i] / Math.sin(Math.atan2(dy[i], dx[i])));

	if (dv > maxVelocity) {
	    double ratio = maxVelocity / dv;
	    dx[i] = (int) (ratio * dx[i]);
	    dy[i] = (int) (ratio * dy[i]);
	}

    }

    public void modHeightData(int gridAmount) {
	for (int i = 0; i <= NUM_GRAINS; i++) {
	    if (isActive[i]) {
		if (y[i] < gridAmount * CELL_SIZE) {
		    isActive[i] = false;
		} else {
		    y[i] %= gridAmount * CELL_SIZE;
		    prevY[i] %= gridAmount * CELL_SIZE;
		    gridY[i] %= gridAmount * CELL_SIZE;
		    prevGridY[i] %= gridAmount * CELL_SIZE;
		}
	    }
	}
    }

    private void emit(int i) {
	isActive[i] = true;
	TTL[i] = GRAIN_LIFE;
	x[i] = x[0];
	y[i] = y[0];
	dx[i] = dx[0];
	dy[i] = dy[0];
	updateVelocity(i, direction
		+ (double) ((Math.PI) + (Math.random() - .5) * EMISSION_ANGLE),
		(int) ((1 + Math.random()) * EMISSION_VELOCITY));
    }

    public void bounce(int i, int impactX, int impactY, boolean reverseX) {
	x[i] = impactX;
	y[i] = impactY;

	if (reverseX) {
	    dx[i] *= -COLLISION_ELASTICITY;
	    dy[i] *= COLLISION_ELASTICITY;
	} else {
	    dx[i] *= COLLISION_ELASTICITY;
	    dy[i] *= -COLLISION_ELASTICITY;
	}
	updateAbsoluteVelocity(i, MAX_GRAIN_VELOCITY);
    }

    private int screenBottom = 0;

    private final int TAIL_LENGTH = 6;

    public void setScreen(CellType[][] screenBuffer) {
	// Get terrain rep
	terrain.setScreen(screenBottom, screenBuffer);

	// Get grain rep

	for (int i = 1; i <= NUM_GRAINS; i++) {
	    if (isActive[i]) {
		safeDrawToScreen(toScreenX(gridX[i]), toScreenY(gridY[i]),
			CellType.GRAIN, screenBuffer);
		// grainCounts[gridX[i]][gridY[i]]++;
	    }
	}

	double vectorX = Math.cos(direction + EMISSION_ANGLE / 2);
	double vectorY = Math.sin(direction + EMISSION_ANGLE / 2);
	double vectorX2 = Math.cos(direction - EMISSION_ANGLE / 2);
	double vectorY2 = Math.sin(direction - EMISSION_ANGLE / 2);
	for (int i = 0; i < TAIL_LENGTH; i++) {
	    safeDrawToScreen(toScreenX(gridX[0]
		    - (int) (vectorX * i * CELL_SIZE) + CELL_SIZE / 2),
		    toScreenY(gridY[0] - (int) (vectorY * i * CELL_SIZE)
			    + CELL_SIZE / 2), CellType.SHIP, screenBuffer);
	    safeDrawToScreen(toScreenX(gridX[0]
		    - (int) (vectorX2 * i * CELL_SIZE) + CELL_SIZE / 2),
		    toScreenY(gridY[0] - (int) (vectorY2 * i * CELL_SIZE)
			    + CELL_SIZE / 2), CellType.SHIP, screenBuffer);
	}
	safeDrawToScreen(toScreenX(gridX[0]), toScreenY(gridY[0]),
		CellType.SHIP_POINT, screenBuffer);
    }

    private int toScreenY(int y) {
	return y / CELL_SIZE - screenBottom;
    }

    private int toScreenX(int x) {
	return x / CELL_SIZE;
    }

    private boolean isOnScreen(int x, int y) {
	return (x >= 0) && (x < GRID_WIDTH) && (y >= 0) && (y < GRID_HEIGHT);
    }

    private void safeDrawToScreen(int x, int y, CellType type,
	    CellType[][] screenBuffer) {
	if (isOnScreen(x, y)) {
	    screenBuffer[x][y] = type;
	}
    }

    // Returns the y value of the end point given the start point, the slope,
    // and the x value of the end point
    private int YgivenX(double slope, float startX, float startY, float endX) {
	return (int) (slope * (endX - startX) + startY);
    }

    // Returns the y value of the end point given the start point, the slope,
    // and the x value of the end point
    private int XgivenY(double slope, float startX, float startY, float endY) {
	return (int) ((endY - startY) / slope + startX);
    }

    private int toGridLoc(int x) {
	return x - x % CELL_SIZE;
    }

    private int toPixelLoc(int x) {
	return x / CELL_SIZE;
    }

    private boolean getCollision(int i) {
	if (prevGridX[i] != gridX[i] || prevGridY[i] != gridY[i]) {
	    double slope = (dy[i] / ((dx[i] == 0) ? 1.0 : dx[i]));
	    // Case: diagonal up
	    if (dy[i] > 0) {
		// Case: rightwards
		int currentX = prevX[i];
		int currentY = prevY[i];
		int currentGridY = toGridLoc(prevY[i]) + CELL_SIZE;

		if (slope > 0) {
		    int currentGridX = toGridLoc(prevX[i]) + CELL_SIZE;

		    while (currentGridX <= (gridX[i] + CELL_SIZE)
			    && currentGridY <= (gridY[i] + CELL_SIZE)) {
			// compare slopes to determine whether to check for
			// intersection with top or right
			if (currentX == currentGridX) {
			    currentX -= 1;
			}
			double slopeToGridCorner = (currentY - currentGridY)
				/ (currentX - currentGridX);
			if (slope > slopeToGridCorner) {
			    // will intersect the top

			    if (terrain.isFull(toPixelLoc(currentGridX
				    - CELL_SIZE), toPixelLoc(currentGridY),
				    screenBottom)) {
				bounce(i,
					XgivenY(slope, currentX, currentY,
						currentGridY), currentGridY,
					false);
				return true;
			    }
			    currentX = XgivenY(slope, currentX, currentY,
				    currentGridY);
			    currentY = currentGridY;
			    currentGridY += CELL_SIZE;
			} else {

			    // will intersect the right side
			    if (terrain.isFull(toPixelLoc(currentGridX),
				    toPixelLoc(currentGridY - CELL_SIZE),
				    screenBottom)) {
				bounce(i,
					currentGridX,
					YgivenX(slope, currentX, currentY,
						currentGridX), true);
				return true;
			    }
			    currentY = YgivenX(slope, currentX, currentY,
				    currentGridX);
			    currentX = currentGridX;
			    currentGridX += CELL_SIZE;

			}
		    }

		} else { // Case: leftwards

		    int currentGridX = toGridLoc(prevX[i]);

		    while (currentGridX >= gridX[i]
			    && currentGridY <= (gridY[i] + CELL_SIZE)) {
			// compare slopes to determine whether to check for
			// intersection with top or right
			if (currentX == currentGridX)
			    currentX += 1;
			double slopeToGridCorner = (currentY - currentGridY)
				/ (currentX - currentGridX);
			if (slope < slopeToGridCorner) {
			    // will intersect the top
			    if (terrain.isFull(toPixelLoc(currentGridX),
				    toPixelLoc(currentGridY), screenBottom)) {
				bounce(i,
					XgivenY(slope, currentX, currentY,
						currentGridY), currentGridY,
					false);
				return true;
			    }

			    currentX = XgivenY(slope, currentX, currentY,
				    currentGridY);
			    currentY = currentGridY;
			    currentGridY += CELL_SIZE;
			} else {
			    // will intersect the left side
			    if (terrain.isFull(toPixelLoc(currentGridX
				    - CELL_SIZE), toPixelLoc(currentGridY
				    - CELL_SIZE), screenBottom)) {
				bounce(i,
					currentGridX,
					YgivenX(slope, currentX, currentY,
						currentGridX), true);
				return true;
			    }
			    currentY = YgivenX(slope, currentX, currentY,
				    currentGridX);
			    currentX = currentGridX;
			    currentGridX -= CELL_SIZE;
			}
		    }
		}
	    } else { // Case: diagonal down
		float currentX = prevX[i];
		float currentY = prevY[i];
		int currentGridY = toGridLoc(prevY[i]);

		// Case: rightwards
		if (slope < 0) {
		    int currentGridX = toGridLoc(prevX[i]) + CELL_SIZE;
		    while (currentGridX <= (gridX[i] + CELL_SIZE)
			    && currentGridY >= gridY[i]) {

			// compare slopes to determine whether to check for
			// intersection with top or right
			if (currentX == currentGridX)
			    currentX -= 1;
			double slopeToGridCorner = (currentY - currentGridY)
				/ (currentX - currentGridX);
			if (slope < slopeToGridCorner) {

			    // will intersect the bottom
			    if (terrain.isFull(toPixelLoc(currentGridX
				    - CELL_SIZE), toPixelLoc(currentGridY
				    - CELL_SIZE), screenBottom)) {

				bounce(i,
					XgivenY(slope, currentX, currentY,
						currentGridY + 1),
					currentGridY, false);
				return true;
			    }
			    currentY = currentGridY;
			    currentGridY -= CELL_SIZE;
			} else {
			    // will intersect the right side
			    if (terrain.isFull(toPixelLoc(currentGridX),
				    toPixelLoc(currentGridY), screenBottom)) {
				bounce(i,
					currentGridX,
					YgivenX(slope, currentX, currentY,
						currentGridX), true);
				return true;
			    }
			    currentY = YgivenX(slope, currentX, currentY,
				    currentGridX);
			    currentX = currentGridX;
			    currentGridX += CELL_SIZE;
			}
		    }
		} else { // Case: leftwards
		    int currentGridX = toGridLoc(prevX[i]);

		    while (currentGridX >= gridX[i] && currentGridY >= gridY[i]) {
			// compare slopes to determine whether to check for
			// intersection with top or right
			if (currentX == currentGridX)
			    currentX += 1;
			double slopeToGridCorner = (currentY - currentGridY)
				/ (currentX - currentGridX);
			if (slope > slopeToGridCorner) {
			    // will intersect the bottom
			    if (terrain.isFull(toPixelLoc(currentGridX),
				    toPixelLoc(currentGridY - CELL_SIZE),
				    screenBottom)) {
				bounce(i,
					XgivenY(slope, currentX, currentY,
						currentGridY),
					currentGridY + 1, false);
				return true;
			    }
			    currentX = XgivenY(slope, currentX, currentY,
				    currentGridY);
			    currentY = currentGridY;
			    currentGridY -= CELL_SIZE;
			} else {
			    // will intersect the left side
			    if (terrain.isFull(toPixelLoc(currentGridX
				    - CELL_SIZE), toPixelLoc(currentGridY),
				    screenBottom)) {
				bounce(i,
					currentGridX,
					YgivenX(slope, currentX, currentY,
						currentGridX), true);
				return true;
			    }
			    currentY = YgivenX(slope, currentX, currentY,
				    currentGridX);
			    currentX = currentGridX;
			    currentGridX -= CELL_SIZE;
			}
		    }
		}
	    }
	}
	return false;
    }
}
