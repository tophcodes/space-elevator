import os
import FreeCAD
import FreeCADGui as Gui

ADDON_DIR = os.path.dirname(__file__)
ICON_PATH = os.path.join(ADDON_DIR, "Resources", "icons", "space-elevator.svg")


class SpaceElevatorWorkbench(Gui.Workbench):
    MenuText = "Space Elevator"
    ToolTip = "6DOF input device integration"
    Icon = ICON_PATH

    def Initialize(self):
        FreeCAD.Console.PrintMessage("Space Elevator: workbench initialized\n")
        from space_elevator import preferences
        preferences.register()

    def Activated(self):
        from space_elevator import lifecycle
        lifecycle.on_activate()

    def Deactivated(self):
        from space_elevator import lifecycle
        lifecycle.on_deactivate()

    def GetClassName(self):
        return "Gui::PythonWorkbench"


Gui.addWorkbench(SpaceElevatorWorkbench())
