"""Space Elevator — thin LCD client for FreeCAD.

No workbench, no spnav. On GUI startup it connects the workbench-activated
signal and pushes the current button bindings to space-elevatord for display
on the SpaceMouse Enterprise LCD. All LCD output is best-effort.
"""

import FreeCAD

try:
    from space_elevator import service
    service.start()
    FreeCAD.Console.PrintMessage("Space Elevator: LCD client active\n")
except Exception as exc:  # never break FreeCAD startup over the LCD
    FreeCAD.Console.PrintWarning(f"Space Elevator: LCD client disabled ({exc})\n")
