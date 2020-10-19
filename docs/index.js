 // Use ES module import syntax to import functionality from the module
// that we have compiled.
//
// Note that the `default` import is an initialization function which
// will "boot" the module and make it ready to use. Currently browsers
// don't support natively imported WebAssembly as an ES module, but
// eventually the manual initialization won't be required!
import init from './rust-web-roguelike.js';

async function run() {
    // First up we need to actually load the wasm file, so we use the
    // default export to inform it where the wasm file is located on the
    // server, and then we wait on the returned promise to wait for the
    // wasm to be loaded.
    // Also note that the promise, when resolved, yields the wasm module's
    // exports which is the same as importing the `*_bg` module in other
    // modes
    await init();
    initRenderer();

}

var term, eng; // Can't be initialized yet because DOM is not ready

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

//init game
run();
//initRenderer();