import colorsys
import json
import math
import struct

from PIL import Image
from websockets.sync.client import connect

def rotate_colour(theta: float, r: int, g: int, b: int, a: int) -> tuple[int, int, int, int]:
    (h, s, v) = colorsys.rgb_to_hsv(r=r / 256, g=g / 256, b=b / 256)
    h += theta
    if h > 2*math.pi:
        h -= 2*math.pi
    rgb = colorsys.hsv_to_rgb(h=h, s=s, v=v)
    return (int(rgb[0] * 256), int(rgb[1] * 256), int(rgb[2] * 256), a)
    
    

def setup_buffer():
    # Get the image
    with open("VIB.png", mode='rb') as png:
        i = Image.open(png)
        assert i.mode == "RGBA"
        return i.tobytes()

def render(rotate_by: float, buf: bytes) -> bytes:
    out = bytearray()
    print(rotate_by)
    for (r, g, b, a) in struct.iter_unpack("BBBB", buf):
        rotated = rotate_colour(rotate_by, r, g, b, a)
        out.extend(rotated)

    return bytes(out)


def main():
    image = setup_buffer()
    rotation = math.pi / 64
    with connect("wss://rse.pagekite.me") as websocket:
        while True:
            rcvd = websocket.recv()
            match rcvd:
                case "?":
                    websocket.send(json.dumps({"msg": "?", "?": "painter"}))
                case "p":
                    data = render(rotation, image)
                    rotation += math.pi /64
                    if rotation > 2 * math.pi:
                        rotation -= 2*math.pi
                    websocket.send(data)
                case _:
                    pass
                


if __name__ == "__main__":
    main()
