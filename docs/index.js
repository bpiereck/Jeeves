const WHO_ARE_YOU = "?";
const SEND_ME_PIXELS = "p";
const CANVAS_SIZE = "size";


const socket = new WebSocket(
	window.location.search.includes("debug")
		? "ws://127.0.0.1:8080"
		: "wss://rse.pagekite.me"

);

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
	console.log(`dim = ${dim}; there are ${pixels.byteLength} bytes in the buffer`);
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

/*
function decodePixels(buffer) {
	const painters = new Map();
	const view = new DataView(buffer);
	const numBuffers = view.getUint16(0);
	const bufferSize = canvas_width * canvas_height * 4;

	for (let i = 0; i < numBuffers; i++) {
		const offset = 2 + (i * (8 + bufferSize));
		const id = view.getBigUint64(offset);
		const pixels = new Uint8ClampedArray(buffer, offset + 8, bufferSize);
		painters.set(id, pixels);
	}
	return painters;
}

function mkDomId(painterId) {
	return `painter-${painterId}`;
}

function difference(existing, from) {
	return existing.filter((k) => !from.has(k.getAttribute("id")));
}

function showOnCanvas(painters) {
	const top = document.getElementById("canvases");
	const existing = Array.from(top.children);

	// remove disconnected painters
	const pids = new Set(Array.from(painters.keys()).map(mkDomId));
	for (const el of difference(existing, pids)) {
		el.remove();
	}

	// add new painters
	const existingIds = new Set(existing.map((c) => c.getAttribute("id")));
	for (const cid of pids.keys()) {
		if (existingIds.has(cid)) {
			continue;
		}
		const el = document.createElement("canvas");
		el.id = cid;
		el.setAttribute("width", `${canvas_width}`);
		el.setAttribute("height", `${canvas_height}`);
		top.appendChild(el);
	}

	// paint pixels
	for (const [id, pixels] of painters) {
		const canvas = document.getElementById(mkDomId(id));
		const ctx = canvas.getContext("2d");
		ctx.imageSmoothingEnabled = false;
		const image = new ImageData(pixels, canvas_height, canvas_width);
		console.log(`Created image data: ${image.width}px x ${image.height}px`);
		ctx.putImageData(image, 0, 0, 0, 0, 2 * image.width, 2 * image.height);
	}
}
*/
