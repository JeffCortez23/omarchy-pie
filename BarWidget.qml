import QtQuick
import Quickshell
import qs.Commons
import qs.Ui

BarWidget {
  id: root
  moduleName: "io.github.jeffcortez23.omarchy-pie"

  readonly property string homeDir: Quickshell.env("HOME")
  readonly property bool hideIcon: setting("hideIcon", false) === true

  visible: !hideIcon
  implicitWidth: hideIcon ? 0 : button.implicitWidth
  implicitHeight: hideIcon ? 0 : button.implicitHeight

  function summonPie() {
    Quickshell.execDetached([root.homeDir + "/.local/bin/omarchy-pie"])
  }

  BarIconButton {
    id: button
    anchors.fill: parent
    bar: root.bar
    text: "󰑮"
    tooltipText: "Omarchy Pie (Super+Z)"
    onPressed: function(buttonCode) {
      if (buttonCode === Qt.LeftButton || buttonCode === Qt.RightButton) {
        root.summonPie()
      }
    }
  }
}
