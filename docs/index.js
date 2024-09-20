const WHO_ARE_YOU = "?";
const SEND_ME_PIXELS = "p";
const CANVAS_SIZE = "size";

const debug = window.location.search.includes("debug");

const socket = new WebSocket(debug ? "ws://127.0.0.1:8080" : "wss://rse.pagekite.me");

socket.addEventListener("message", async (event) => {
	if (event.data instanceof Blob) {
			event.data.arrayBuffer().then(showOnCanvas);
	} else {
		const message = JSON.parse(event.data);
		switch (message.msg) {
		case WHO_ARE_YOU:
			socket.send(JSON.stringify({msg: WHO_ARE_YOU, [WHO_ARE_YOU]: "canvas"}));
			break;
		default:
			break;
		}
	}
});

function showOnCanvas(pixels) {
	const canvas = document.getElementById("canvas");
	const bounds = canvas.getBoundingClientRect();
	const ctx = canvas.getContext("2d");
	ctx.imageSmoothingEnabled = false;

	const view = new DataView(pixels);
	const dim = view.getUint16(0, false);
	canvas.width = dim;
	canvas.height = dim;
	if (debug) {
		console.log(`dim = ${dim}; there are ${pixels.byteLength} bytes in the buffer`);
		console.log(pixelsToHex(new Uint8Array(pixels)));
	}
	const bufferSize = dim * dim * 4;
	if (dim > 0) {
		const pixeldata = new Uint8ClampedArray(pixels, 2, bufferSize)
		const image = new ImageData(pixeldata, dim, dim);
		ctx.putImageData(image, 0, 0, 0, 0, bounds.width, bounds.height);
	} else {
		// clear the canvas
		console.log("Clearing the canvas");
		ctx.clearRect(0, 0, canvas.width, canvas.height);
	}
}

function pixelsToHex(data) {
	return [...data].map((x) => x.toString(16).padStart(2, '0')).join('');
}

// periodically ask for pixels
setInterval(() => {
	if (socket.readyState === 1) {
		socket.send(JSON.stringify({msg: SEND_ME_PIXELS}));
	} else {
		console.warn(`Socket.readyState = ${socket.readyState}`);
	}
}, 1000);

