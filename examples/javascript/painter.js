const socket = new WebSocket("wss://rse.pagekite.me");

const WHO_ARE_YOU = "?";
const SEND_ME_PIXELS = "p";
const BUFFER_SIZE = "size";

const buffer = {
	data: new Uint8Array(0),
	width: 0,
	height: 0,
};

function randomByte() {
	return Math.floor(Math.random() * 255);
}

function randomPixel() {
	return Math.floor(Math.random() * (buffer.width * buffer.height * 4) - 4);
}

// once a second modify some pixels
// don't do this when the request for pixels comes
// in so no time is wasted sending pixel data back
setInterval(() => {
	if (buffer.width > 0 && buffer.height > 0) {
		const pixel = randomPixel();
		console.log(`Changing pixel: ${pixel}`);
		buffer.data[pixel] = randomByte();
		buffer.data[pixel + 1] = randomByte();
		buffer.data[pixel + 2] = randomByte();
	}
}, 1000);

socket.addEventListener("message", (event) => {
	const message = JSON.parse(event.data);
	switch (message.msg) {
	case WHO_ARE_YOU:
		socket.send(JSON.stringify({
			msg: WHO_ARE_YOU,
			[WHO_ARE_YOU]: "painter",
			name: "MaybeJustJames",
			url: "https://james.thecolliers.xyz"
		}));
		break;

	case SEND_ME_PIXELS:
		socket.send(buffer.data);
  	break;

	case BUFFER_SIZE:
		buffer.width = message.w;
		buffer.height = message.h;
		buffer.data = new Uint8Array(buffer.width * buffer.height * 4);
		break;

	default:
		break;
	}
})
