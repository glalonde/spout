package spout;

import static spout.GlobalConstants.FPS;
import static spout.GlobalConstants.GRID_HEIGHT;
import static spout.GlobalConstants.GRID_WIDTH;
import static spout.GlobalConstants.SCALE;
import static spout.GlobalConstants.WINDOWS_WINDOW_OFFSET_VERTICAL;
import static spout.GlobalConstants.WINDOWS_WINDOW_OFFSET_HORIZONTAL;
import static spout.GlobalConstants.OSX_WINDOW_OFFSET_VERTICAL;
import static spout.GlobalConstants.OSX_WINDOW_OFFSET_HORIZONTAL;

import java.awt.Canvas;
import java.awt.Color;
import java.awt.Graphics2D;
import java.awt.GraphicsConfiguration;
import java.awt.GraphicsEnvironment;
import java.awt.Toolkit;
import java.awt.Transparency;
import java.awt.event.KeyEvent;
import java.awt.event.KeyListener;
import java.awt.event.WindowAdapter;
import java.awt.event.WindowEvent;
import java.awt.image.BufferStrategy;
import java.awt.image.BufferedImage;

import javax.swing.JFrame;
import javax.swing.WindowConstants;

public class Spout extends Thread {
    private boolean isRunning = true;
    private Canvas canvas;
    private BufferStrategy strategy;
    private BufferedImage background;
    private Graphics2D backgroundGraphics;
    private Graphics2D graphics;
    private JFrame frame;

    private GraphicsConfiguration config = GraphicsEnvironment
	    .getLocalGraphicsEnvironment().getDefaultScreenDevice()
	    .getDefaultConfiguration();

    // create a hardware accelerated image
    public final BufferedImage create(final int width, final int height,
	    final boolean alpha) {
	return config.createCompatibleImage(width, height,
		alpha ? Transparency.TRANSLUCENT : Transparency.OPAQUE);
    }

    // Game objects, and grid sizing
    private Game game = new Game();
    private boolean keyExhaust = false;
    private boolean keyTurnLeft = false;
    private boolean keyTurnRight = false;

    // Setup
    public Spout() {
    	
	String OS = System.getProperty("os.name").toLowerCase();
	int WINDOW_OFFSET_VERTICAL;
	int WINDOW_OFFSET_HORIZONTAL;
	
	if(OS.indexOf("win") >= 0) {
		WINDOW_OFFSET_VERTICAL = WINDOWS_WINDOW_OFFSET_VERTICAL;
		WINDOW_OFFSET_HORIZONTAL = WINDOWS_WINDOW_OFFSET_HORIZONTAL;
	} else if (OS.indexOf("mac") >= 0) {
		WINDOW_OFFSET_VERTICAL = OSX_WINDOW_OFFSET_VERTICAL;
		WINDOW_OFFSET_HORIZONTAL = OSX_WINDOW_OFFSET_HORIZONTAL;
	} else {
		WINDOW_OFFSET_VERTICAL = WINDOWS_WINDOW_OFFSET_VERTICAL;
		WINDOW_OFFSET_HORIZONTAL = WINDOWS_WINDOW_OFFSET_HORIZONTAL;
	}

	// JFrame
	frame = new JFrame();
	frame.addWindowListener(new FrameClose());

	frame.setDefaultCloseOperation(WindowConstants.DO_NOTHING_ON_CLOSE);
	frame.setSize(GRID_WIDTH * SCALE + WINDOW_OFFSET_HORIZONTAL,
		GRID_HEIGHT * SCALE + WINDOW_OFFSET_VERTICAL);
	frame.setVisible(true);
	frame.setResizable(false);

	// Canvas
	canvas = new Canvas(config);
	canvas.setSize(GRID_WIDTH * SCALE, GRID_HEIGHT * SCALE);
	canvas.addKeyListener(new KeyListener() {
	    @Override
	    public void keyPressed(KeyEvent e) {
		switch (e.getKeyCode()) {
		case (KeyEvent.VK_Z):
		case (KeyEvent.VK_UP):
		    keyExhaust = true;
		    break;
		case (KeyEvent.VK_LEFT):
		    keyTurnLeft = true;
		    break;
		case (KeyEvent.VK_RIGHT):
		    keyTurnRight = true;
		    break;
		}
	    }

	    @Override
	    public void keyReleased(KeyEvent e) {
		switch (e.getKeyCode()) {
		case (KeyEvent.VK_Z):
		case (KeyEvent.VK_UP):
		    keyExhaust = false;
		    break;
		case (KeyEvent.VK_LEFT):
		    keyTurnLeft = false;
		    break;
		case (KeyEvent.VK_RIGHT):
		    keyTurnRight = false;
		    break;
		}
	    }

	    @Override
	    public void keyTyped(KeyEvent e) {

	    }

	});
	frame.add(canvas, 0);

	// Background & Buffer
	background = create(GRID_WIDTH, GRID_HEIGHT, false);
	canvas.createBufferStrategy(2);
	do {
	    strategy = canvas.getBufferStrategy();
	} while (strategy == null);

	start();
    }

    private class FrameClose extends WindowAdapter {
	@Override
	public void windowClosing(final WindowEvent e) {
	    isRunning = false;
	}
    }

    // Screen and buffer stuff
    private Graphics2D getBuffer() {
	if (graphics == null) {
	    try {
		graphics = (Graphics2D) strategy.getDrawGraphics();
	    } catch (IllegalStateException e) {
		return null;
	    }
	}
	return graphics;
    }

    private boolean updateScreen() {
	graphics.dispose();
	graphics = null;
	try {
	    strategy.show();
	    Toolkit.getDefaultToolkit().sync();
	    return (!strategy.contentsLost());

	} catch (NullPointerException e) {
	    return true;

	} catch (IllegalStateException e) {
	    return true;
	}
    }

    @Override
    public void run() {
	backgroundGraphics = (Graphics2D) background.getGraphics();
	long fpsWait = (long) (1.0 / FPS * 1000);
	backgroundGraphics.setColor(CellType.EMPTY.color);
	backgroundGraphics.fillRect(0, 0, GRID_WIDTH, GRID_HEIGHT);
	main: while (isRunning) {
	    long renderStart = System.nanoTime();
	    if (game.isOver) {
		if (keyExhaust && (keyTurnLeft || keyTurnRight)) {
		    game = new Game();
		} else {
		    game.step(false, false, false);
		}
	    } else {
		game.step(keyExhaust, keyTurnLeft, keyTurnRight);
	    }

	    // Update Graphics
	    do {
		Graphics2D bg = getBuffer();
		if (!isRunning) {
		    break main;
		}
		renderGame(backgroundGraphics); // this calls your draw method
		// thingy
		if (SCALE != 1) {
		    bg.drawImage(background, 0, 0, GRID_WIDTH * SCALE,
			    GRID_HEIGHT * SCALE, 0, 0, GRID_WIDTH, GRID_HEIGHT,
			    null);
		} else {
		    bg.drawImage(background, 0, 0, null);
		}
		bg.dispose();
	    } while (!updateScreen());

	    // Better do some FPS limiting here
	    long renderTime = (System.nanoTime() - renderStart) / 1000000;
	    try {
		Thread.sleep(Math.max(0, fpsWait - renderTime));
	    } catch (InterruptedException e) {
		Thread.interrupted();
		break;
	    }
	    renderTime = (System.nanoTime() - renderStart) / 1000000;

	}
	frame.dispose();
    }

    public void renderGame(Graphics2D g) {
	// render grid
	for (int x = 0; x < GRID_WIDTH; x++) {
	    for (int y = 0; y < GRID_HEIGHT; y++) {
		if (game.deltaBuffer[x][y]) {
		    g.setColor(game.screenBuffer[x][y].color);
		    g.fillRect(x, GRID_HEIGHT - y - 1, 1, 1);
		}
	    }
	}

	if (game.isOver) {
	    g.setColor(Color.RED);
	    g.drawString("GAME OVER", GRID_WIDTH / 2 - 30, GRID_HEIGHT / 2);
	}
	g.setColor(Color.DARK_GRAY);
	g.drawString("" + game.mobileObjects.score, 0, GRID_HEIGHT - 1);
    }

    public static void main(final String args[]) {
	new Spout();
    }
}