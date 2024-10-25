from galactic import GalacticUnicorn
from network_manager import NetworkManager
from picographics import PicoGraphics, DISPLAY_GALACTIC_UNICORN as DISPLAY
from pngdec import PNG
from time import sleep
import CONFIG
import asyncio
import uurequests

CURRENT_PNG = "current.png"

# overclock to 200Mhz
machine.freq(200000000)

# create galactic object and graphics surface for drawing
galactic = GalacticUnicorn()
graphics = PicoGraphics(DISPLAY)

BLACK = graphics.create_pen(0, 0, 0)
WHITE = graphics.create_pen(155, 155, 155)

def clear_display():
    graphics.set_pen(BLACK)
    graphics.clear()
    graphics.set_pen(WHITE)

def draw_text(text, line):
    if line == 0:
        y = -1
    else:
        y = 5

    graphics.text(text, 0, y, -1, 1, 0)

brightness = 0.5
galactic.set_brightness(brightness)
clear_display()
graphics.set_font("bitmap6")

draw_text("starting", 0)
galactic.update(graphics)

sleep(2)

def status_handler(mode, status, ip):
    clear_display()

    top_text = "connecting"
    bottom_text = "{}".format(CONFIG.SSID)

    if status is not None:
        if status:
            top_text = "success"
            bottom_text = "{}".format(ip)
        else:
            top_text = "failed"

    draw_text(top_text, 0)
    draw_text(bottom_text, 1)
    galactic.update(graphics)

network_manager = NetworkManager(CONFIG.COUNTRY, status_handler=status_handler)
asyncio.get_event_loop().run_until_complete(network_manager.client(CONFIG.SSID, CONFIG.PSK))

sleep(3)

clear_display()
draw_text("server", 0)
draw_text("{}".format(CONFIG.SERVER_ADDR), 1)
galactic.update(graphics)

sleep(3)

url = "http://{}/current.png".format(CONFIG.SERVER_ADDR)

while True:
    print("fetching {}".format(url))
    response = uurequests.get(url, parse_headers = True)
    print("fetched")

    clear_display()

    png = PNG(graphics)
    png.open_RAM(response.content)
    png.decode(0, 0)

    galactic.update(graphics)

    sleep(15)
