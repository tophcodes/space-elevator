import os
import FreeCADGui as Gui

_UI_PATH = os.path.join(os.path.dirname(__file__), "..", "Resources", "ui", "preferences.ui")


def register():
    Gui.addPreferencePage(os.path.abspath(_UI_PATH), "Space Elevator")
