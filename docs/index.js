const WHO_ARE_YOU = "?";
const SEND_ME_PIXELS = "p";
const CANVAS_SIZE = "size";

const socket = new WebSocket("wss://rse.pagekite.me");

let canvas_width = 0;
let canvas_height = 0;

socket.addEventListener("message", async (event) => {
	if (event.data instanceof Blob) {
			event.data.arrayBuffer().then(decodePixels).then(showOnCanvas);
	} else {
		const message = JSON.parse(event.data);
		switch (message.msg) {
		case WHO_ARE_YOU:
			socket.send(JSON.stringify({msg: WHO_ARE_YOU, [WHO_ARE_YOU]: "canvas"}));
			break;
		case CANVAS_SIZE:
			canvas_width = message.w;
			canvas_height = message.h;
			break;
		default:
			break;
		}
	}
});

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


// periodically ask for pixels
setInterval(() => {
	console.log(socket.readyState);
	socket.send(JSON.stringify({msg: SEND_ME_PIXELS}));
}, 1000);
