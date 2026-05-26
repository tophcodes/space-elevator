"""Apply transformed SpaceMouse motion to the active FreeCAD 3D view.

Strategy: translate the camera in view-local coordinates for tx/ty/tz, rotate
about the camera's local axes for rx/ry/rz. Period comes from libspnav in ms;
we use it as a per-event timestep so feel is consistent regardless of report
rate.
"""

import FreeCADGui as Gui

# Tunables — exposed via Settings in a later iteration if needed.
PAN_UNITS_PER_FULL_DEFLECTION_PER_SECOND = 60.0   # mm/s at full stick
ROT_RADIANS_PER_FULL_DEFLECTION_PER_SECOND = 1.2  # rad/s at full stick


def _active_camera():
    doc = Gui.ActiveDocument
    if doc is None:
        return None
    view = doc.ActiveView
    if view is None:
        return None
    return view.getCameraNode()


def apply(transformed_axes, period_ms):
    cam = _active_camera()
    if cam is None:
        return
    tx, ty, tz, rx, ry, rz = transformed_axes
    dt = max(period_ms, 1) / 1000.0

    pan_factor = PAN_UNITS_PER_FULL_DEFLECTION_PER_SECOND * dt
    rot_factor = ROT_RADIANS_PER_FULL_DEFLECTION_PER_SECOND * dt

    _translate(cam, tx * pan_factor, ty * pan_factor, tz * pan_factor)
    _rotate(cam, rx * rot_factor, ry * rot_factor, rz * rot_factor)


def _translate(cam, dx, dy, dz):
    # Camera position lives in world space. Use the orientation to map view-
    # local (dx, dy, dz) into world deltas.
    orient = cam.orientation.getValue()
    world_delta = orient.multVec((dx, dy, dz))
    pos = cam.position.getValue().getValue()
    cam.position.setValue(pos[0] + world_delta[0], pos[1] + world_delta[1], pos[2] + world_delta[2])


def _rotate(cam, rx, ry, rz):
    # Compose three small-angle rotations about camera-local axes.
    from pivy import coin
    cur = cam.orientation.getValue()
    if rx:
        cur = cur * coin.SbRotation(coin.SbVec3f(1, 0, 0), rx)
    if ry:
        cur = cur * coin.SbRotation(coin.SbVec3f(0, 1, 0), ry)
    if rz:
        cur = cur * coin.SbRotation(coin.SbVec3f(0, 0, 1), rz)
    cam.orientation.setValue(cur)
