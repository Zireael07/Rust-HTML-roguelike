//JS starts here
import * as rust from './rust_web_roguelike.js';

//JS Lisp implementation
import {res} from './mal.js';

var term, eng, inventoryOverlay, vendorOverlay; // Can't be initialized yet because DOM is not ready
var universe, g_wasm, map, player, entities_mem,w,h; // Can't be initialized yet because WASM is not ready

// The tile palette is precomputed in order to not have to create
// thousands of Tiles on the fly.
var AT = new ut.Tile("@", 255, 255, 255);
var THUG = new ut.Tile("t", 55, 0, 0, 255, 0, 0); //red bg means hostile
var KNIFE = new ut.Tile("/", 0, 255, 255);
var MED = new ut.Tile("!", 255, 0, 0);
var BARKEEP = new ut.Tile("☺", 0, 128, 255, 255, 255, 0); //yellow bg means neutral

var TABLE = new ut.Tile("╦", 170, 170, 170);
var CHAIR = new ut.Tile("└", 170, 170, 170);

var WALL = new ut.Tile('▒', 100, 100, 100);
var FLOOR = new ut.Tile('.', 50, 50, 50);
var GRASS = new ut.Tile(',', 0, 255, 0);
var TREE = new ut.Tile('♣', 0, 153, 0);
var FLOOR_INDOOR = new ut.Tile('.', 0, 128, 128);
var DOOR = new ut.Tile("+", 211, 211, 211);

//JS stub logic starts here

//absolutely needs to match Rust logic in map.rs!!!
const getIndex = (x, y) => {
    return y * w + x;
};

//inverse of the above
const getPos = (ind) => {
    return [ind % w, ind / w];
}

//wrappers for Rust functions
function is_Visible(x,y){
    return universe.is_visible(x,y);
}

function isSeen(x,y) {
    return universe.is_seen(x,y);
}

function shouldDraw(x,y) {
    return universe.should_draw(x,y);
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
    if (v == 2 ) { return GRASS };
    if (v == 3 ) { return TREE };
    if (v == 4 ) { return FLOOR_INDOOR};
    if (v == 5 ) { return DOOR};
  	
	if (t === '#') return WALL;
	if (t === '.') return FLOOR;
	return ut.NULLTILE;
}

function getRenderTile(x,y) {
	if (is_Visible(x,y)) {
		return getDungeonTile(x,y)
	}
	else if (isSeen(x,y)) {
		var tile = getDungeonTile(x,y);
		if (tile == ut.NULLTILE) { 
			return ut.NULLTILE //those don't have anything to tint xDDD
		} 
		else {
			return new ut.Tile(tile.getChar(), tile.r*0.5, tile.g*0.5, tile.b*0.5)
		}
	}
	//paranoia
	else {
		return ut.NULLTILE;
	}
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
        if (tile == 0) { tile = THUG; }
        if (tile == 1) { tile = KNIFE; }
        if (tile == 2) { tile = MED; }
        if (tile == 3) { tile = BARKEEP; }
        if (tile == 4) { tile = TABLE; }
        if (tile == 5) { tile = CHAIR; }

		// if (e.tile == null || e.tile == undefined) {
		// 	console.log("Tile for " + e + " is null!");
		// 	continue;
		// }
		term.put(tile, tilex, tiley);
    }
    
    term.render(); // Render

}

//inventory
function clickFunction(button) {
    //extract id from item id
    var id = button.id;
    var reg = id.match(/(\d+)/); 
    var i = reg[0];
    //console.log("ID: ", i);
    inventoryOverlay.setVisibility(false); //close the inventory
    var item = universe.inventory_items()[i];
	console.log("Pressed button " + button.innerHTML, " id: ", item);
	universe.use_item_ext(item);
}

function display_name(item){
    return universe.inventory_name_for_id(item);
}

//based on redblobgames
function createInventoryOverlay() {
    const overlay = document.querySelector("#inventory");
    let visible = false;

    function draw() {
        let html = `<ul>`;
        let empty = true;

        //let len = player.inventory.items.length;
        let len = universe.inventory_size();
		for (var i = 0; i < len; ++i) {
            //var item = player.inventory.items[i];
            var item = universe.inventory_items()[i];
            html += `<li><button class="inv_button" id=item-${i}>${String.fromCharCode(65 + i)}</button> ${display_name(item)}</li>`;
			empty = false;
			//not added yet!
			//var button = document.querySelector(".inv_button");
        } //);
        html += `</ul>`;
        if (empty) {
            html = `<div>Your inventory is empty. Press <kbd>I</kbd> again to cancel.</div>${html}`;
        } else {
            html = `<div>Select an item to use it, or <kbd>I</kbd> again to cancel.</div>${html}`;
        }
		overlay.innerHTML = html;
		//TODO: fold into the previous somehow?
		for (var i = 0; i < len; i++) {
			//var buttons = document.querySelectorAll(".inv_button");
			//for (var i=0; i < buttons.length; ++i) {
                //var button = buttons[i];
                var button = document.querySelector('#item-'+CSS.escape(i));
                //var item = universe.inventory_items()[i];
                //console.log(button, item);
                //anonymous function
				button.onclick = function(e) { clickFunction(e.target); }
			//}
		}
    }

    return {
        get visible() { return visible; },
        setVisibility(visibility) {
            visible = visibility;
            overlay.classList.toggle('visible', visibility);
            if (visible) draw();
        },
    };
}


function showInventory() {
	//var set = inventoryOverlay.visible? false : true;
	if (inventoryOverlay.visible) {
		inventoryOverlay.setVisibility(false);
	}
	else if (!inventoryOverlay.visible) {
		inventoryOverlay.setVisibility(true);
	}
	//return;
}

// vendor 
function vendorClick(button) {
    //extract id from item id
    var id = button.id;
    var reg = id.match(/(\d+)/); 
    var i = reg[0];
    //console.log("ID: ", i);
    vendorOverlay.classList.toggle('visible', false); //close the listing
    universe.change_money(5.0);
    universe.give_item("Protein shake");
    //var item = universe.inventory_items()[i];
	console.log("Pressed vendor button " + button.innerHTML); //, " id: ", item);
}

function creationSelect(el) {
    //console.log("ID: " + el.id + " " + el.selectedIndex);
    // check for duplicates
    if (el.selectedIndex != 0) {
        var lines = document.getElementById("creation").getElementsByTagName("li");
        for (var i = 0, len = lines.length; i < len; i++ ) {
            if (i != el.id) {
                //console.log("Other: " + i + " " + lines[i].children[0].selectedIndex);
                if (lines[i].children[0].selectedIndex == el.selectedIndex) {
                    lines[i].children[0].selectedIndex = 0 //set it to --
                }
            }
        }
    }

}

function confirmCreation() {
    var allow = true;
    var lines = document.getElementById("creation").getElementsByTagName("li");
    var stats = [];
    for (var i = 0, len = lines.length; i < len; i++ ) {
        var sel = lines[i].children[0] 
        if (sel.selectedIndex == 0) {
            allow = false;
        }
        else {
            stats[i] = sel.options[sel.selectedIndex].text;
        }
    }

    if (allow) {
        document.getElementById("creation").classList.toggle('visible', false); //close the listing
        //assign the stats
        universe.set_player_stats(stats);
        //handle post-start
        universe.on_game_start();
    }
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
    else if (k == ut.KEY_I) {
        if (!vendorOverlay.classList.contains('visible')) {
            cmd = rust.Command.Inventory //dummy
            showInventory() //do our thing
        }

    } 
    else if (k == ut.KEY_S) {
        cmd = rust.Command.SaveGame; //dummy
        let save = universe.save_game();
        // unfortunately we're using 0.0.6...
        let storage = new Sifrr.Storage();
        let data = {'save': save};
        storage.set(data).then(() => {
            console.log("Saved game to browser...");
        }) ;
    }
    else if (k == ut.KEY_R) //'R'estore because L is taken by 'vikeys'
    {
        cmd = rust.Command.SaveGame; //dummy
        let storage = new Sifrr.Storage(); //with the same (default) options, we access the same storage
        storage.get('save').then(value => {
            console.log("Loaded game: ", value)
            //pass to Rust
            universe.load_save(value.save);
        });
    }
    else if (k == 27) // escape
    {
        if (vendorOverlay.classList.contains('visible')) {
            vendorOverlay.classList.toggle('visible', false); //close the listing
        }
    }

    if (cmd != -1) {
        // update Rust
        universe.process(cmd);
        // update display
        tick();
    }

}

function initRenderer(wasm) {
    universe = rust.Universe.new();
    // those are the map tiles, they don't change
    map = universe.get_tiles();
    player = universe.player();
    g_wasm = wasm;

    w = universe.width();
    h = universe.height();

    window.setInterval(tick, 50); // Animation
	// Initialize Viewport, i.e. the place where the characters are displayed
	term = new ut.Viewport(document.getElementById("game"), 40, 25, "dom");
	// Initialize Engine, i.e. the Tile manager
    eng = new ut.Engine(term, getRenderTile, w, h); //w,h
    //use fov
    eng.setMaskFunc(shouldDraw);

    //more game init
    inventoryOverlay = createInventoryOverlay();
    vendorOverlay = document.getElementById("vendor");
    //anonymous function
    vendorOverlay.firstElementChild.onclick = function(e) { vendorClick(e.target); }

	// Initialize input
    ut.initInput(onKeyDown);
    
    //handle post-start
    //universe.on_game_start();
    // character creation screen
    document.getElementById("creation").classList.toggle('visible', true);
    var lines = document.getElementById("creation").getElementsByTagName("li");
    for (var i = 0, len = lines.length; i < len; i++ ) {
        lines[i].children[0].onchange = function(e) { creationSelect(e.target); }
    }
    var button = document.getElementById("confirm")
    button.onclick = function(e) { confirmCreation() }

    //test JS Lisp
    var line = "{ + 4 {* 3 4} }"
    res(line);

    universe.spawn(player[0]+2, player[1]+2, "Thug");

}

export { initRenderer }