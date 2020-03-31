package spout;

public abstract class GlobalConstants {
    // Speed governing constants, represents both frames per second and game
    // state updates per second
    public static final int FPS = 60;
    
    
    public static final int WINDOWS_WINDOW_OFFSET_VERTICAL = 38;
    public static final int WINDOWS_WINDOW_OFFSET_HORIZONTAL = 16;
    public static final int OSX_WINDOW_OFFSET_VERTICAL = 22;
    public static final int OSX_WINDOW_OFFSET_HORIZONTAL = 0;

    // Dimensional environment constants
    public static final int CELL_SIZE = (int) Math.pow(2, 16);
    public static final int GRID_WIDTH = 256;
    public static final int GRID_HEIGHT = 160;

    public static final int SCALE = 3;

    public static final int LEVEL_HEIGHT = 2 * GRID_HEIGHT;
    public static final int SCROLL_HEIGHT = GRID_HEIGHT / 2;

    // Level design constants
    public static final int MAX_VACANCY_SIZE = Math
	    .min(GRID_WIDTH, GRID_HEIGHT) * 2 / 3;

    // Ship constants
    public static final int MAX_SHIP_VELOCITY = 96 * CELL_SIZE / FPS;
    public static final int MAX_GRAIN_VELOCITY = MAX_SHIP_VELOCITY * 3;
    public static final double TURN_VELOCITY = 6.0 / FPS;

    // 500 ms to max velocity
    public static final int EXHAUST_ACCEL = (1000 / 500) * MAX_SHIP_VELOCITY
	    / (FPS);
    // public static final int EXHAUST_ACCEL = 0;

    // Physics Environment constants
    public static final double GRAVITY = -MAX_SHIP_VELOCITY / (3 * FPS);
    // public static final float GRAVITY = 0;
    public static final double COLLISION_ELASTICITY = .5;

    // Grain constants
    public static final int EMISSION_VELOCITY = (int) (1.5 * MAX_SHIP_VELOCITY);
    public static final double EMISSION_ANGLE = Math.PI / 5;
    public static final int EMISSION_RATE = 18;
    public static final int GRAIN_LIFE = 2 * FPS;
    public static final int NUM_GRAINS = GRAIN_LIFE * EMISSION_RATE;

}
