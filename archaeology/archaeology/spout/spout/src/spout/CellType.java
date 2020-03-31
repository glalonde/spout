package spout;

import java.awt.Color;

public enum CellType {
    GROUND(Color.GRAY), SHIP(Color.BLACK), SHIP_POINT(Color.magenta), TAIL(
	    Color.RED), GRAIN(Color.ORANGE), EMPTY(Color.WHITE);

    public final Color color;

    private CellType(Color color) {
	this.color = color;
    }
}
