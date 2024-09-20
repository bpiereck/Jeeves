"""Example painter in Python."""

import colorsys
from collections.abc import Callable
from dataclasses import dataclass
import json
import math
import struct
from types import SimpleNamespace

from PIL import Image
from websockets.sync.client import connect

MESSAGES = SimpleNamespace()
MESSAGES.WHO_ARE_YOU = "?"
MESSAGES.SEND_ME_PIXELS = "p"
MESSAGES.BUFFER_SIZE = "size"
MESSAGES.ERROR = "error"

Pixel = tuple[int, int, int, int]

@dataclass(frozen=True)
class Buffer:
    """Image data to transform."""
    image: list[Pixel]
    width: int
    height: int

def rotate_colour(theta: float, r: int, g: int, b: int, a: int) -> tuple[int, int, int, int]:
    """Convert (Red, Green, Blue) colour space to (Hue, Saturation, Value). The Hue is encoded
    as an angle in the range [0, 2*pi]. Rotate the Hue by `theta` then convert back to RGB.
    Keep the opacity (`a`) the same."""
    (h, s, v) = colorsys.rgb_to_hsv(r=r / 255, g=g / 255, b=b / 255)
    h += theta
    if h > 2*math.pi:
        h -= 2*math.pi
    rgb = colorsys.hsv_to_rgb(h=h, s=s, v=v)
    return (int(rgb[0] * 255), int(rgb[1] * 255), int(rgb[2] * 255), a)
    

def setup_buffer(width: int, height: int) -> Buffer:
    """Get the image binary data."""
    with open("VIB_200_200.png", mode='rb') as png:
        i = Image.open(png)
        assert i.mode == "RGBA"
        if i.width > width or i.height > height:
            i = i.resize(size=(width, height), resample=Image.Resampling.LANCZOS)

        return Buffer(image=list(struct.iter_unpack("BBBB", i.tobytes())),
                      height=i.height,
                      width=i.width)

def render(rotate_by: float, buf: Buffer) -> bytes:
    """Rotate every pixels hue."""
    # Check that the buffer has been initialised
    if buf.height > 0 and buf.width > 0:
        out = bytearray()
        for (r, g, b, a) in buf.image:
            rotated = rotate_colour(rotate_by, r, g, b, a)
            out.extend(rotated)

        return bytes(out)
    else:
        print("Not initialized")
        return b''


def main():
    """Main entry point."""
    import sys
    if len(sys.argv) >= 2:
        ws_uri = sys.argv[1]
    else:
        ws_uri = "wss://rse.pagekite.me"
    image = Buffer(image=b'', width=0, height=0)
    rotation = 0
    with connect(ws_uri) as websocket:
        while True:
            text = websocket.recv()
            message = json.loads(text)
            match message.get("msg"):
                case MESSAGES.WHO_ARE_YOU:
                    websocket.send(json.dumps({
                        "msg": MESSAGES.WHO_ARE_YOU,
                        f"{MESSAGES.WHO_ARE_YOU}": "painter",
                        "name": "James Collier",
                        "url": "https://github.com/MaybeJustJames"
                    }))
                case MESSAGES.SEND_ME_PIXELS:
                    data = render(rotation, image)
                    rotation += math.pi / 128
                    if rotation > 2 * math.pi:
                        rotation -= 2*math.pi
                    websocket.send(data)
                case MESSAGES.BUFFER_SIZE:
                    width = message.get("w")
                    height = message.get("h")
                    image = setup_buffer(width, height)
                case MESSAGES.ERROR:
                    print(message.get(MESSAGES.ERROR))
                case other:
                    print("Unknown message:")
                    print(other)
                    print("+++++++++++++++++++++")
                


if __name__ == "__main__":
    main()
