"""Renode capture sequence for Spirograph.

Run from xous-dev-toolkit:
    python3 scripts/renode_capture.py --app spirograph --script ../precursor-spirograph/capture.py

Contract: run(ctl, outdir) -> number of failed steps. The app is already launched.
"""
import os


def run(ctl, outdir) -> int:
    failures = 0

    def shot(name):
        nonlocal failures
        if not ctl.try_screenshot(os.path.join(outdir, name)):
            failures += 1

    shot("01_menu.png")
    for _ in range(4):
        ctl.timed_key("Down", after=1.0)
    ctl.inject_line("")  # Enter on Draw: the pen starts tracing
    ctl.sleep(1.0)
    shot("02_tracing.png")
    ctl.sleep(6.0)
    shot("03_finished.png")
    ctl.timed_key("BackSpace", after=1.0)
    ctl.timed_key("Down", after=1.0)
    ctl.inject_line("")  # Enter on Gallery
    ctl.sleep(1.0)
    shot("04_gallery.png")
    return failures
