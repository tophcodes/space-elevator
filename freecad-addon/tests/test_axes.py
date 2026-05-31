from space_elevator.axes import transform


def test_deadzone_suppresses_small_values():
    raw = (5, 5, 5, 5, 5, 5)  # inside 0.02 * 350 = 7
    scales = (1.0,) * 6
    inverts = (False,) * 6
    out = transform(raw, scales=scales, inverts=inverts, deadzone=0.02, max_raw=350)
    assert out == (0.0,) * 6


def test_scale_applied_after_deadzone():
    raw = (350, 0, 0, 0, 0, 0)
    out = transform(raw, scales=(2.0, 1, 1, 1, 1, 1), inverts=(False,)*6, deadzone=0.02, max_raw=350)
    assert abs(out[0] - 2.0) < 1e-6  # normalised to 1.0, then *2


def test_invert_flips_sign():
    raw = (350, 0, 0, 0, 0, 0)
    out = transform(raw, scales=(1.0,)*6, inverts=(True, False, False, False, False, False), deadzone=0.0, max_raw=350)
    assert out[0] == -1.0
