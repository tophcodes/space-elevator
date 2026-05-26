import ctypes

from space_elevator.libspnav import (
    _Event,
    _MotionEvent,
    _ButtonEvent,
    SPNAV_EVENT_MOTION,
    SPNAV_EVENT_BUTTON,
)


def test_event_union_size_matches_motion_struct():
    # Union must be at least as large as the largest member.
    assert ctypes.sizeof(_Event) >= ctypes.sizeof(_MotionEvent)
    assert ctypes.sizeof(_Event) >= ctypes.sizeof(_ButtonEvent)


def test_event_type_field_alias():
    ev = _Event()
    ev.motion.type = SPNAV_EVENT_MOTION
    assert ev.type == SPNAV_EVENT_MOTION
    ev.button.type = SPNAV_EVENT_BUTTON
    assert ev.type == SPNAV_EVENT_BUTTON


def test_motion_struct_field_order():
    m = _MotionEvent(type=1, x=10, y=-20, z=30, rx=1, ry=2, rz=3, period=16, data=None)
    assert m.x == 10 and m.y == -20 and m.z == 30
