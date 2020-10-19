var term, eng; // Can't be initialized yet because DOM is not ready

// "Main loop"
function tick() {
}

function initRenderer() {
    window.setInterval(tick, 50); // Animation
	// Initialize Viewport, i.e. the place where the characters are displayed
	term = new ut.Viewport(document.getElementById("game"), 40, 25, "dom");
	// Initialize Engine, i.e. the Tile manager
	//eng = new ut.Engine(term, getDungeonTile, 20, 20);
	// Initialize input
	//ut.initInput(onKeyDown);
}