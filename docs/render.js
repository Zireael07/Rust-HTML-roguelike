//JS starts here
import * as rust from './rust_web_roguelike.js';

//JS Lisp implementation
import {res} from './mal.js';

var term, eng, inventoryOverlay, vendorOverlay, viewOverlay,logOverlay; // Can't be initialized yet because DOM is not ready
var universe, g_wasm, map, player, entities_mem,w,h; // Can't be initialized yet because WASM is not ready
var mouse = null
var automoving = false;

// The tile palette is precomputed in order to not have to create
// thousands of Tiles on the fly.
var AT = new ut.Tile("@", 255, 255, 255);
var THUG = new ut.Tile("t", 55, 0, 0); //, 255, 0, 0); //red bg means hostile
var KNIFE = new ut.Tile("/", 0, 255, 255);
var MED = new ut.Tile("!", 255, 0, 0);
var BARKEEP = new ut.Tile("☺", 0, 128, 255); //, 255, 255, 0); //yellow bg means neutral
var PATRON = new ut.Tile("☺", 100, 100, 100); //, 255, 255, 0);

var BOOTS = new ut.Tile("]", 129, 77, 4, 255,255,255);
var JACKET = new ut.Tile("]", 255,124,0, 255,255,255);
var JEANS = new ut.Tile("]", 0, 23, 255, 255,255,255);


var TABLE = new ut.Tile("╦", 170, 170, 170);
var CHAIR = new ut.Tile("└", 170, 170, 170);
var BED = new ut.Tile("#", 0, 128, 128);

var WALL = new ut.Tile('▒', 100, 100, 100);
var FLOOR = new ut.Tile('.', 50, 50, 50);
var GRASS = new ut.Tile(',', 0, 255, 0);
var TREE = new ut.Tile('♣', 0, 153, 0);
var FLOOR_INDOOR = new ut.Tile('.', 0, 128, 128);
var DOOR = new ut.Tile("+", 211, 211, 211);
var WATER = new ut.Tile("~", 0, 0, 255);
var MOUNTAIN = new ut.Tile("▓", 200, 200, 200); //▲ fits some sort of rubble and # is more fitting for a web imho

//JS stub logic starts here

//absolutely needs to match Rust logic in map.rs!!!
const getIndex = (x, y) => {
    return y * w + x;
};

//inverse of the above
const getPos = (ind) => {
    return [Math.round(ind % w), Math.round(ind / w) ];
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
    if (v == 6 ) { return MOUNTAIN};
    if (v == 7 ) { return WATER};
  	
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
        if (tile == 6 ) { tile = BOOTS};
        if (tile == 7 ) { tile = JACKET};
        if (tile == 8 ) { tile = JEANS};
        if (tile == 9) { tile = PATRON};
        if (tile == 10) { tile = BED};

		// if (e.tile == null || e.tile == undefined) {
		// 	console.log("Tile for " + e + " is null!");
		// 	continue;
        // }
        
        //mark attitude/faction with background color
        if (tile == THUG || tile == BARKEEP || tile == PATRON) {
           var fact = universe.get_faction(ex, ey);
           if (fact == 0) {    
               term.put(new ut.Tile(tile.ch, tile.r, tile.g, tile.b, 255, 0, 0), tilex, tiley); //red bg means hostile
           }
           if (fact == 1) {
               term.put(new ut.Tile(tile.ch, tile.r, tile.g, tile.b, 255, 255, 0), tilex, tiley); //yellow bg means neutral
           }
        }
        else {
            term.put(tile, tilex, tiley);            
        }

		//term.put(tile, tilex, tiley);
    }
    
    // draw highlight under clicked tile
    if (mouse) {
        var t = term.get(mouse.x, mouse.y);
        //dark highlight (one of the default colors offered by CSS picker)
        term.put(new ut.Tile(t.ch, t.r, t.g, t.b, 63, 81, 181), mouse.x, mouse.y);
    }

    // draw player AFTER everything else
    term.put(AT, term.cx, term.cy); // Player character centered for free by JS

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

function dropclickFunc(button) {
    //extract id from item id
    var id = button.id;
    var reg = id.match(/(\d+)/); 
    var i = reg[0];
    //console.log("ID: ", i);
    inventoryOverlay.setVisibility(false); //close the inventory
    var item = universe.inventory_items()[i];
    console.log("Pressed drop button " + button.innerHTML, " id: ", item);
    universe.drop_item_ext(item);
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
            html += `<li><button class="inv_button" id=item-${i}>${String.fromCharCode(65 + i)}</button> ${display_name(item)}<button class="drop_button" id=item-drop-${i}>d</button></li>`;
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
                var dropb = document.querySelector('#item-drop-'+CSS.escape(i));
                dropb.onclick = function(e) { dropclickFunc(e.target); }
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

function examineFunction(button){
    //extract id from item id
    var id = button.id;
    var reg = id.match(/(\d+)/); 
    var i = reg[0];
    console.log("ID: ", i);
    document.getElementById("description").classList.toggle('visible', true);
    //var w_pos = worldPos(mouse);
    var w_pos = universe.entity_view_pos(i);
    //draw
    let html = "<div>DESCRIPTION</div><p style='white-space: pre-line'>"; //this style makes HTML understand \n linebreaks
    html += universe.get_description(w_pos[0], w_pos[1]) + '</p> <p>\nPress E again to close</p>';
    document.getElementById("description").innerHTML = html;
}


//view listing
function createViewListOverlay() {
    const overlay = document.querySelector("#viewlist");
    let visible = false;

    function draw() {
        let html = `<div>VIEW LISTING</div><ul>`;
        let empty = true;

        let viewlist = universe.view_list();

        let len = viewlist.length;
		for (var i = 0; i < len; ++i) {
            var item = viewlist[i];
            html += `<li> ${universe.view_string_for_id(item)} <button id=examine-${item}>E</button></li>`;
			empty = false;
        } //);
        html += `</ul>`;
        // if (empty) {
        //     html = `<div>Your inventory is empty. Press <kbd>I</kbd> again to cancel.</div>${html}`;
        // } else {
        //     html = `<div>Select an item to use it, or <kbd>I</kbd> again to cancel.</div>${html}`;
        // }
        overlay.innerHTML = html;

        //TODO: fold into the previous somehow?
		for (var i = 0; i < len; i++) {
                var item = viewlist[i];
                var button = document.querySelector('#examine-'+CSS.escape(item));
                //anonymous function
                button.onclick = function(e) { examineFunction(e.target); }
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


function showViewList() {
	//var set = inventoryOverlay.visible? false : true;
	if (viewOverlay.visible) {
		viewOverlay.setVisibility(false);
	}
	else if (!viewOverlay.visible) {
		viewOverlay.setVisibility(true);
	}
	//return;
}

function showDescription() {
    if (!document.getElementById("description").classList.contains('visible')) {
        document.getElementById("description").classList.toggle('visible', true);
        var w_pos = worldPos(mouse);
        //draw
        let html = "<div>DESCRIPTION</div><p style='white-space: pre-line'>"; //this style makes HTML understand /n linebreaks
        html += universe.get_description(w_pos.x, w_pos.y) + '</p>';
        document.getElementById("description").innerHTML = html;
    }
    else {
        document.getElementById("description").classList.toggle('visible', false); //close the listing
    }
        
}

//log view
function showLogHistory() {
	if (!document.getElementById("log-history").classList.contains('visible')) {
		document.getElementById("log-history").classList.toggle('visible', true);
	}
	// else {
	// 	document.getElementById("log-history").classList.toggle('visible', false); //close the listing
	// }
}

function showRestView() {
    if (!document.getElementById("rest").classList.contains('visible')) {
        document.getElementById("rest").classList.toggle('visible', true);
    }
}

function waitClick(button) {
    //extract id from item id
    var id = button.id;
    var reg = id.match(/(\d+)/); 
    var i = reg[0];

    var tim = -1;
    //lookup
    if (i == 1) { tim = rust.WaitType.Minutes5 }
    if (i == 2) { tim = rust.WaitType.Minutes30 }
    if (i == 3) { tim = rust.WaitType.Hour1 }
    if (i == 4) { tim = rust.WaitType.Hour2 }
    if (i == 5) { tim = rust.WaitType.TillDusk } 

    if (tim != -1) {
        universe.wait(tim);
        //hide list
        document.getElementById("rest").classList.toggle('visible', false);
    }
    
}

//tabs
function openTab(evt) {
    // Declare all variables
    var i, tabcontent, tablinks;

    //Get tabname
    var tabName = evt.target.id.replace('tab-', '');
  
    // Get all elements with class="tabcontent" and hide them
    tabcontent = document.getElementsByClassName("tabcontent");
    for (i = 0; i < tabcontent.length; i++) {
      tabcontent[i].style.display = "none";
    }
  
    // Get all elements with class="tablinks" and remove the class "active"
    tablinks = document.getElementsByClassName("tablinks");
    for (i = 0; i < tablinks.length; i++) {
      tablinks[i].className = tablinks[i].className.replace(" active", "");
    }
  
    // Show the current tab, and add an "active" class to the button that opened the tab
    document.getElementById(tabName).style.display = "block";
    evt.currentTarget.className += " active";
  }

// Key press handler - movement & collision handling
//Just converts to rust commands
function onKeyDown(k) {
    var cmd = -1;
	if (k === ut.KEY_LEFT || k === ut.KEY_H) cmd = rust.Command.MoveLeft;
	else if (k === ut.KEY_RIGHT || (k === ut.KEY_L && !ut.isKeyPressed(ut.KEY_SHIFT))) cmd = rust.Command.MoveRight;
	else if (k === ut.KEY_UP || k === ut.KEY_K) cmd = rust.Command.MoveUp;
    else if (k === ut.KEY_DOWN || k === ut.KEY_J) cmd = rust.Command.MoveDown;
    else if (k == ut.KEY_G) cmd = rust.Command.GetItem;
    else if (k == ut.KEY_R && !ut.isKeyPressed(ut.KEY_SHIFT)) cmd = rust.Command.Rest;
    else if (k == ut.KEY_PERIOD) //'r' is taken by 'rest' above 
    {
        cmd = rust.Command.Wait; // dummy
        showRestView();
    }
    else if (k == ut.KEY_I) {
        if (!vendorOverlay.classList.contains('visible')) {
            cmd = rust.Command.Inventory //dummy
            showInventory() //do our thing
        }

    }
    else if (k == ut.KEY_E) {
        showDescription()
    }
    else if (k == ut.KEY_V) {
        showViewList()
    } 
    //the usual way would be event.shiftkey but ut exposes only (k)eycodes, not the whole event
    else if (k == ut.KEY_L && ut.isKeyPressed(ut.KEY_SHIFT)) {
        showLogHistory();
        console.log("Pressed Shift+L");
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
    else if (k == ut.KEY_R && ut.isKeyPressed(ut.KEY_SHIFT)) //'R'estore because L is taken by 'vikeys'
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
        if (logOverlay.classList.contains('visible')) {
            document.getElementById("log-history").classList.toggle('visible', false); //close the listing
        }
        if (document.getElementById("conversation").classList.contains('visible')) {
            document.getElementById("conversation").classList.toggle('visible', false); //close the conversation
        }
    }

    //if blocking screens are open, ignore keys
    if (document.getElementById("creation").classList.contains('visible') || 
    vendorOverlay.classList.contains('visible') || logOverlay.classList.contains('visible') || 
    document.getElementById("conversation").classList.contains('visible') || document.getElementById("inventory").classList.contains('visible')) {
        //console.log("Ignoring command because blocking screen is open...");
        cmd = -1;
    }

    if (cmd != -1) {
        // update Rust
        universe.process(cmd);
        // update display
        tick();
    }

}

//mouse/touch
function getMousePos(e) {
    return {x:e.clientX,y:e.clientY};
}

function relPos(e, gm) {
	return {x: e.clientX-gm.offsetLeft, y: e.clientY-gm.offsetTop};
}


function termPos(e, gm) {
	var rel = relPos(e, gm);
	//hack
	var gm_s = gm.getBoundingClientRect();
	var tile_w = (gm_s.width)/term.w;
	var tile_h = (gm_s.height)/term.h;
	//console.log(tile_w + " " + tile_h);
	var tx = Math.floor(rel.x/tile_w);
	var ty = Math.floor(rel.y/tile_h);

	//term.tw and term.th should be set by DOMRenderer's updateStyle() but it's not :(
	return {x: tx, y: ty}
}


function worldPos(t_pos){
	//console.log("Term pos: x" + t_pos.x + "y: " + t_pos.y);
	// term.cx and term.cy always == player position
	// this comes out to top left coordinates
	var cam_x = player[0]-term.cx;
	var cam_y = player[1]-term.cy;
	//console.log("Cam pos: x: " + cam_x + "y: " + cam_y);
	return {x: t_pos.x+cam_x, y: t_pos.y+cam_y}
}

function onClickH(w_pos) {
	//ignore clicks outside of map
	if (w_pos.x < 0 || w_pos.y < 0 || w_pos.x > w || w_pos.y > h) {
		return;
	} 

    var dir_x = w_pos.x-player[0]
    var dir_y = w_pos.y-player[1]

	//move player
	if (dir_x < 2 && dir_x > -2 && dir_y < 2 && dir_y > -2){
        universe.move_player(dir_x, dir_y);
	} else {
        // store target and/or path on Rust side
        universe.astar_path(w_pos.x, w_pos.y);
    }
	tick();
}

//logic shuffled to Rust (see load_datafiles())
//needs to be async to be able to use await
async function initGame(wasm) {
    universe = rust.Universe.new();
    //async/await again to load text data
    //workaround
    universe = await rust.load_datafile_ex(universe);
//     const res = await fetch("./npcs.ron");
//     //console.log(res);
//     const ron = await res.text();
//     console.log(ron);
     initRenderer(wasm);
}


function initRenderer(wasm) {
    //universe = rust.Universe.new();
    //workaround
    //universe = await rust.load_datafile();

    //rust.load_datafile(universe); //we can't use things created from Rust this way as it'll cause null pointer error

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
    //init UI stuff
    inventoryOverlay = createInventoryOverlay();
    viewOverlay = createViewListOverlay();
    vendorOverlay = document.getElementById("vendor");
    logOverlay = document.getElementById("log-history");
    //anonymous function
    vendorOverlay.firstElementChild.onclick = function(e) { vendorClick(e.target); }
    var c = document.getElementById("tabs").children
    for (i = 0; i < c.length; i++) {
        c[i].onclick = function(e) {openTab(e) } //, c[i].id.replace('tab-', '')); }
    }

    //default to ASCII map open
    document.getElementById("game").style.display = "block";

    var c = document.getElementById("rest").children
    for (i = 0; i < c.length; i++) {
        c[i].onclick = function(e) { waitClick(e.target) }
    }

	// Initialize input
    ut.initInput(onKeyDown);
    
    // mouse and touch input
	var gm = document.getElementById("game");
	gm.addEventListener('mousedown', e => { 
		e.preventDefault();
		//var m_pos = getMousePos(e);
		//console.log("Pressed mouse @ x: " + m_pos.x + " y: " + m_pos.y);
		//var r_pos = relPos(e, gm);
		//console.log("Position relative to gm: x: " + r_pos.x + " y:" + r_pos.y);
		mouse = termPos(e, gm);
		//console.log("Term pos: x: " + t_pos.x + " y: " + t_pos.y);
		var w_pos = worldPos(mouse);
		//console.log("World pos: x " + w_pos.x + " y: " + w_pos.y);
		onClickH(w_pos);
	});
    gm.addEventListener('mouseup', e => { e.preventDefault() } );

    //mouse move triggers proceeding with stuff
    // this allows the player to effectively pause the game if he's not interacting
	gm.addEventListener('mousemove', e => { 
		e.preventDefault();
		mouse = termPos(e, gm);
        //console.log(mouse);

        //trigger automove
        var steps = universe.get_automove();
        if (universe.has_automove() && steps.length > 1 && !automoving) {
            console.log("We have automove...");
            automoving = true;
            setTimeout(function() { 
                //alert("After 1 seconds!");
                //pop the first step
                var pos = universe.get_pos(steps[0])
                var dir_x = pos[0]-player[0]
                var dir_y = pos[1]-player[1]
                if (dir_x == 0 && dir_y == 0) {
                    automoving = true; //abort
                }
                universe.move_player(dir_x, dir_y);
                universe.advance_automove();
                //redraw
                tick();
                automoving = false;
            }, 1000); //1 second
        }


        // in case we have something to process on the Rust side, send a dummy command...
        universe.process(rust.Command.None);
        //to redraw the highlight
        tick();

	});

    //hidden console
    document.getElementById("console-input").addEventListener('keyup', e => {
        //console.log("Keyup:", e.target.value);
        e.stopPropagation();
    });
    document.getElementById("console-input").addEventListener('keydown', e => {
        //console.log("Keydown:", e.target.value);
        e.stopPropagation();
    });
    //only fires when the value is committed
    document.getElementById("console-input").addEventListener('change', e => {
        //console.log("Console input", e.target.value);
        universe.console_input(e.target.value);
        e.target.value = null //empty the input after accepting it
    });

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

    // //test JS Lisp
    // var line = "{ + 4 {* 3 4} }"
    // res(line);

    // universe.spawn_ex(player[0]+2, player[1]+2, "Patron");

}

export { initGame }