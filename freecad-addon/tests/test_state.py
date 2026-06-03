from space_elevator import state


def test_build_state_splits_left_right_and_blanks_unbound():
    names = [f"Cmd{i}" if i not in (3, 7) else "" for i in range(12)]

    def read_bindings():
        return names

    def resolve(name):
        return {"label": name.upper(), "icon": f"<svg>{name}</svg>"}

    def active_wb():
        return "SketcherWorkbench"

    result = state.build_state(read_bindings=read_bindings, resolve=resolve, active_wb=active_wb)

    assert result["profile"] == "FreeCAD"
    assert result["mode"] == "SketcherWorkbench"
    assert len(result["left"]) == 6 and len(result["right"]) == 6

    # bound tile carries label + icon
    assert result["left"][0] == {"label": "CMD0", "icon": "<svg>Cmd0</svg>", "active": False}
    # unbound index 3 -> blank tile
    assert result["left"][3] == {"label": "", "icon": None, "active": False}
    # right cluster starts at index 6; index 7 unbound
    assert result["right"][1] == {"label": "", "icon": None, "active": False}


def test_build_state_unknown_resolve_none_uses_name():
    def read_bindings():
        return ["Mystery"] + [""] * 11

    def resolve(name):
        return {"label": name, "icon": None}

    result = state.build_state(read_bindings=read_bindings, resolve=resolve, active_wb=lambda: "PartWorkbench")
    assert result["left"][0] == {"label": "Mystery", "icon": None, "active": False}
