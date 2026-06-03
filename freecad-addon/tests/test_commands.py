from space_elevator import commands


def test_read_icon_rasterizes_to_png_data_uri():
    def icon_factory(path):
        assert path == ":/icons/zoom-all.svg"
        return "QUJD"  # fake base64

    icon = commands.read_icon("zoom-all", icon_factory=icon_factory)
    assert icon == "data:image/png;base64,QUJD"


def test_read_icon_none_when_no_pixmap():
    assert commands.read_icon("", icon_factory=lambda p: None) is None


def test_read_icon_none_when_resource_missing():
    assert commands.read_icon("nope", icon_factory=lambda p: None) is None


def test_resolve_strips_accelerator_and_attaches_icon():
    class FakeCmd:
        def getInfo(self):
            return {"menuText": "&Fit all", "pixmap": "zoom-all"}

    def command_get(name):
        assert name == "Std_ViewFitAll"
        return FakeCmd()

    info = commands.resolve(
        "Std_ViewFitAll",
        command_get=command_get,
        icon_resolver=lambda px: "<svg/>",
    )
    assert info == {"label": "Fit all", "icon": "<svg/>"}


def test_resolve_unknown_command_uses_name_as_label():
    info = commands.resolve(
        "Bogus_Cmd",
        command_get=lambda name: None,
        icon_resolver=lambda px: None,
    )
    assert info == {"label": "Bogus_Cmd", "icon": None}
