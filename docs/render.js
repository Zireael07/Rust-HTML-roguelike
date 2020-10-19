//JS starts here
import * as rust from './rust-web-roguelike.js';

var term, eng; // Can't be initialized yet because DOM is not ready
var universe, g_wasm; // Can't be initialized yet because WASM is not ready

// The tile palette is precomputed in order to not have to create
// thousands of Tiles on the fly.
var AT = new ut.Tile("@", 255, 255, 255);
var WALL = new ut.Tile('▒', 100, 100, 100);
var FLOOR = new ut.Tile('.', 50, 50, 50);


//JS stub logic starts here

const getIndex = (x, y) => {
    return x * 20 + y;
};

// Returns a Tile based on the char array map
function getDungeonTile(x, y) {
    var t = "";
    var v = -1;
	try { 
        //t = map[y][x];

        const cells = universe.get_cells();
        const idx = getIndex(x, y);
        v = cells[idx];
        console.log("Cell at ", x, " y: ", y, "is: ", v);
    }
    catch(err) { return ut.NULLTILE; }

    //map rust values to our tiles
    if (v == 1 ) { t = '.'};
    if (v == 2 ) { t = '#'};
  	
	if (t === '#') return WALL;
	if (t === '.') return FLOOR;
	return ut.NULLTILE;
}

function tick() {
    eng.update(1, 1); // Update tiles
	term.put(AT, term.cx, term.cy); // Player character
	term.render(); // Render
}

function initRenderer(wasm) {
    universe = rust.Universe.new();
    g_wasm = wasm;

    window.setInterval(tick, 50); // Animation
	// Initialize Viewport, i.e. the place where the characters are displayed
	term = new ut.Viewport(document.getElementById("game"), 40, 25, "dom");
	// Initialize Engine, i.e. the Tile manager
	eng = new ut.Engine(term, getDungeonTile, 20, 20);
	// Initialize input
	//ut.initInput(onKeyDown);
}

export { initRenderer }