from galactic import GalacticUnicorn
from network_manager import NetworkManager
from picographics import PicoGraphics, DISPLAY_GALACTIC_UNICORN as DISPLAY
from pngdec import PNG
from time import sleep
from strptime import strptime
import CONFIG
import asyncio
import uurequests

CURRENT_PNG = "current.png"
HTTP_DATE = "%a, %d %b %Y %H:%M:%S GMT"
STARTUP_DELAY = 2

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

def is_leap(year):
    return year % 4 == 0 and (year % 100 != 0 or year % 400 == 0)

MONTH_DAYS = (
    0,
    31,
    28,
    31,
    30,
    31,
    30,
    31,
    31,
    30,
    31,
    30,
    31
)

def days_for(year, month, day):
    _year = year - 1
    year_days = _year * 365 + _year // 4 - _year // 100 + _year // 400

    month_days = MONTH_DAYS[month] + (month > 2 and is_leap(year))

    return year_days + month_days + day

EPOCH_DATE = days_for(1970, 1, 1)

def epoch_seconds(time):
    days = days_for(time.tm_year, time.tm_mon, 1) - EPOCH_DATE + time.tm_yday - 2

    hours = 24 * days + time.tm_hour

    minutes = 60 * hours + time.tm_min

    return 60 * minutes + time.tm_sec

brightness = 0.5
galactic.set_brightness(brightness)
clear_display()
graphics.set_font("bitmap6")

draw_text("starting", 0)
galactic.update(graphics)

sleep(STARTUP_DELAY)

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

sleep(STARTUP_DELAY)

clear_display()
draw_text("server", 0)
draw_text("{}".format(CONFIG.SERVER_ADDR), 1)
galactic.update(graphics)

sleep(STARTUP_DELAY)

url = "http://{}/current.png".format(CONFIG.SERVER_ADDR)

while True:
    print("fetching {}".format(url))
    response = uurequests.get(url, parse_headers = True)

    date = response.headers["date"]
    date = epoch_seconds(strptime(date, HTTP_DATE))

    expires = response.headers["expires"]
    expires = epoch_seconds(strptime(expires, HTTP_DATE))

    next_update = expires - date

    print("fetched, next update: {}".format(next_update))

    clear_display()

    png = PNG(graphics)
    png.open_RAM(response.content)
    png.decode(0, 0)

    galactic.update(graphics)

    sleep(next_update)
