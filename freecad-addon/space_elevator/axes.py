def transform(raw, scales, inverts, deadzone, max_raw=350):
    """Map a 6-tuple of raw libspnav axes to scaled floats in roughly [-N, N].

    raw: tuple of six ints (x, y, z, rx, ry, rz)
    scales / inverts: 6-tuples
    deadzone: fraction of max_raw under which values clamp to 0
    max_raw: raw axis range — libspnav reports roughly +-350 for SpaceMouse Enterprise

    Returns a tuple of six floats. A value of 1.0 means "axis at full deflection".
    """
    dz = deadzone * max_raw
    out = []
    for v, s, inv in zip(raw, scales, inverts):
        if -dz < v < dz:
            out.append(0.0)
            continue
        norm = v / max_raw
        if inv:
            norm = -norm
        out.append(norm * s)
    return tuple(out)
