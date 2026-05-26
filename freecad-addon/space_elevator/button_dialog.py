try:
    from PySide6 import QtWidgets
except ImportError:
    from PySide2 import QtWidgets

import FreeCAD
import FreeCADGui as Gui

from . import buttons


N_BUTTONS = 31  # SpaceMouse Enterprise has 31 mappable buttons


class BindingsDialog(QtWidgets.QDialog):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Space Elevator — Button bindings")
        self.resize(520, 600)

        self._bindings = buttons.Bindings(FreeCAD.ParamGet)
        self._command_names = sorted(Gui.listCommands())

        layout = QtWidgets.QVBoxLayout(self)

        table = QtWidgets.QTableWidget(N_BUTTONS, 2, self)
        table.setHorizontalHeaderLabels(["Button", "Command"])
        table.horizontalHeader().setStretchLastSection(True)
        for i in range(N_BUTTONS):
            table.setItem(i, 0, QtWidgets.QTableWidgetItem(f"#{i}"))
            combo = QtWidgets.QComboBox()
            combo.addItem("(none)", "")
            for name in self._command_names:
                combo.addItem(name, name)
            current = self._bindings.get(i) or ""
            idx = combo.findData(current)
            if idx >= 0:
                combo.setCurrentIndex(idx)
            table.setCellWidget(i, 1, combo)
        self._table = table
        layout.addWidget(table)

        buttons_box = QtWidgets.QDialogButtonBox(
            QtWidgets.QDialogButtonBox.Save | QtWidgets.QDialogButtonBox.Cancel
        )
        buttons_box.accepted.connect(self._save)
        buttons_box.rejected.connect(self.reject)
        layout.addWidget(buttons_box)

    def _save(self):
        for i in range(N_BUTTONS):
            combo = self._table.cellWidget(i, 1)
            self._bindings.set(i, combo.currentData())
        self.accept()


def show():
    dlg = BindingsDialog(Gui.getMainWindow())
    dlg.exec_()
