//JS starts here
import * as rust from './rust-web-roguelike.js';

var term, eng; // Can't be initialized yet because DOM is not ready
var universe, g_wasm, map, player, entities_mem; // Can't be initialized yet because WASM is not ready

// The tile palette is precomputed in order to not have to create
// thousands of Tiles on the fly.
var AT = new ut.Tile("@", 255, 255, 255);
var THUG = new ut.Tile("t", 255, 0, 0);
var KNIFE = new ut.Tile("/", 0, 255, 255);

var WALL = new ut.Tile('▒', 100, 100, 100);
var FLOOR = new ut.Tile('.', 50, 50, 50);


//JS stub logic starts here

const getIndex = (x, y) => {
    return x * 20 + y;
};

//inverse of the above
const getPos = (ind) => {
    return [ind % 20, ind / 20];
}

//wrapper for Rust function
function is_Visible(x,y){
    return universe.is_visible(x,y);
}

// Returns a Tile based on the char array map
function getDungeonTile(x, y) {
    var t = "";
    var v = -1;
	try { 
        //t = map[y][x];
       // const map = universe.get_cells(); 
        const idx = getIndex(x, y);
        v = map[idx];
        //console.log("Cell at ", x, " y: ", y, "is: ", v);
    }
    catch(err) { return ut.NULLTILE; }

    //map rust values to our tiles
    if (v == 0 ) { return FLOOR };
    if (v == 1 ) { return WALL };
  	
	if (t === '#') return WALL;
	if (t === '.') return FLOOR;
	return ut.NULLTILE;
}

// Main loop
function tick() {
    var i, len, ex, ey, tile, tilex, tiley; //cache
    player = universe.player();

    //player is always centered (see below); cx is half width
	//so this comes out to top left coordinates
	var cam_x = player[0]-term.cx;
	var cam_y = player[1]-term.cy;

    eng.update(player[0], player[1]); // Update tiles in viewport
    term.put(AT, term.cx, term.cy); // Player character centered for free by JS
    
    //draw entities
    entities_mem = universe.draw_entities();
    len = entities_mem.length;
    for (i = 0; i < len; i += 3) {
        ex = entities_mem[i + 0];
        ey = entities_mem[i + 1]
        tile = entities_mem[i + 2];
        //console.log("x:", ex, "y:", ey, "glyph:", tile);

        //draw in screen space
		tilex = ex - cam_x;
        tiley = ey - cam_y;
        //substitute correct glyph
        if (tile == 0) {
            tile = THUG;
        }
        if (tile == 1) {
            tile = KNIFE;
        }

		// if (e.tile == null || e.tile == undefined) {
		// 	console.log("Tile for " + e + " is null!");
		// 	continue;
		// }
		term.put(tile, tilex, tiley);
    }
    
    term.render(); // Render

}

// Key press handler - movement & collision handling
//Just converts to rust commands
function onKeyDown(k) {
    var cmd = -1;
	if (k === ut.KEY_LEFT || k === ut.KEY_H) cmd = rust.Command.MoveLeft;
	else if (k === ut.KEY_RIGHT || k === ut.KEY_L) cmd = rust.Command.MoveRight;
	else if (k === ut.KEY_UP || k === ut.KEY_K) cmd = rust.Command.MoveUp;
    else if (k === ut.KEY_DOWN || k === ut.KEY_J) cmd = rust.Command.MoveDown;
    else if (k == ut.KEY_G) cmd = rust.Command.GetItem;
    
    // update Rust
    universe.process(cmd);
    // update display
	tick();
}

function initRenderer(wasm) {
    universe = rust.Universe.new();
    // those are the map tiles, they don't change
    map = universe.get_tiles();
    player = universe.player();
    g_wasm = wasm;

    window.setInterval(tick, 50); // Animation
	// Initialize Viewport, i.e. the place where the characters are displayed
	term = new ut.Viewport(document.getElementById("game"), 40, 25, "dom");
	// Initialize Engine, i.e. the Tile manager
    eng = new ut.Engine(term, getDungeonTile, 20, 20);
    //use fov
    eng.setMaskFunc(is_Visible);
	// Initialize input
	ut.initInput(onKeyDown);
}

export { initRenderer }