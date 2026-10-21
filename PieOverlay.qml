import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import QtQuick
import qs.Commons
import qs.Commons as Commons
import qs.Ui

Item {
  id: root

  property string omarchyPath: Quickshell.env("OMARCHY_PATH")
  property string homePath: Quickshell.env("HOME")
  property var shell: null
  property var manifest: null

  property bool opened: false
  property int hoveredIndex: -1
  property real wheelScale: 0.85

  // File context properties
  property string filePath: ""
  property string fileName: ""
  property string fileSize: ""
  property string fileKind: ""
  property string fileIcon: "󰈔"
  property bool isImage: false
  property string dimensions: ""
  property var rootSlices: []
  property var slices: []
  property bool inSubMenu: false
  property string subMenuTitle: ""

  // Cursor & positioning
  property int cursorX: 960
  property int cursorY: 540
  property int monitorWidth: 1920
  property int monitorHeight: 1080
  readonly property int wheelSize: 450
  readonly property int outerRadius: 206
  readonly property int innerRadius: 82

  // QR Mode state
  property bool qrMode: false
  property bool qrLoading: false
  property string qrUrl: ""
  property string qrSvgPath: ""
  property string qrSubtitle: ""

  // Theme colors inherited from Omarchy
  property color bgColor: Commons.Color.menu.background
  property color fgColor: Commons.Color.menu.text
  property color borderColor: Commons.Color.menu.border
  property color accentColor: Commons.Color.accent
  property color urgentColor: Commons.Color.urgent
  property color mutedColor: Commons.Color.muted
  property color selectedBg: Commons.Color.menu.selectedBackground
  property color selectedFg: Commons.Color.menu.selectedText
  property color scrimColor: Commons.Color.menu.scrim
  property string fontFamily: Style.font.menuFamily

  onBgColorChanged: pieCanvas.requestPaint()
  onFgColorChanged: pieCanvas.requestPaint()
  onAccentColorChanged: pieCanvas.requestPaint()
  onHoveredIndexChanged: pieCanvas.requestPaint()
  onSlicesChanged: pieCanvas.requestPaint()

  function resolveBinaryPath() {
    if (root.manifest && root.manifest.__sourceDir) {
      return String(root.manifest.__sourceDir).replace(/\/$/, "") + "/bin/omarchy-pie"
    }
    return root.homePath + "/.local/bin/omarchy-pie"
  }

  function open(payloadJson) {
    root.qrMode = false
    root.qrLoading = false
    root.qrUrl = ""
    root.qrSvgPath = ""
    root.hoveredIndex = -1
    root.inSubMenu = false
    root.subMenuTitle = ""
    root.wheelScale = 0.84

    if (payloadJson && typeof payloadJson === "string" && payloadJson.length > 2) {
      try {
        var data = JSON.parse(payloadJson)
        root.filePath = String(data.file_path || "")
        root.fileName = String(data.file_name || "archivo")
        root.fileSize = String(data.file_size || "")
        root.fileKind = String(data.file_kind || "ARCHIVO")
        root.fileIcon = String(data.file_icon || "󰈔")
        root.isImage = Boolean(data.is_image)
        root.dimensions = String(data.dimensions || "")
        root.cursorX = Number(data.cursor_x) || 960
        root.cursorY = Number(data.cursor_y) || 540
        root.monitorWidth = Number(data.monitor_width) || 1920
        root.monitorHeight = Number(data.monitor_height) || 1080
        root.rootSlices = Array.isArray(data.slices) ? data.slices : []
        root.slices = root.rootSlices
      } catch (e) {
        console.warn("omarchy-pie: failed to parse payload:", e)
      }
    }

    root.opened = true
    root.wheelScale = 1.0
    pieCanvas.requestPaint()
    Qt.callLater(function() { keyCatcher.forceActiveFocus() })
  }

  function goBack() {
    if (root.inSubMenu) {
      root.inSubMenu = false
      root.subMenuTitle = ""
      root.slices = root.rootSlices
      root.hoveredIndex = -1
      pieCanvas.requestPaint()
      return true
    }
    return false
  }

  function close() {
    root.opened = false
    root.qrMode = false
    root.qrLoading = false
    root.inSubMenu = false
  }

  function dismiss() {
    root.close()
    if (root.shell && typeof root.shell.hide === "function") {
      root.shell.hide((root.manifest && root.manifest.id) || "io.github.jeffcortez23.omarchy-pie")
    }
  }

  function toggle(payloadJson) {
    if (root.opened) root.dismiss()
    else root.open(payloadJson || "{}")
  }

  function showQr(argJson) {
    try {
      var info = JSON.parse(argJson)
      root.qrUrl = String(info.url || "")
      root.qrSvgPath = String(info.qrPath || "")
      root.qrSubtitle = String(info.subtitle || "Scan QR code")
      root.qrLoading = false
      root.qrMode = true
    } catch (e) {
      console.warn("omarchy-pie: showQr parse error:", e)
    }
    return "ok"
  }

  function triggerSlice(index) {
    if (index < 0 || index >= root.slices.length) return
    var s = root.slices[index]
    if (!s || !s.id) return

    // Sub-menu transition
    if (Array.isArray(s.sub_slices) && s.sub_slices.length > 0) {
      root.subMenuTitle = String(s.label || "").replace(" ▸", "")
      root.slices = s.sub_slices
      root.inSubMenu = true
      root.hoveredIndex = -1
      pieCanvas.requestPaint()
      return
    }

    var binPath = root.resolveBinaryPath()
    var actionId = String(s.id)
    var targetFile = root.filePath
    if (actionId === "share_qr" || actionId === "upload_0x0") {
      root.qrMode = true
      root.qrLoading = true
      root.qrSubtitle = "Preparando enlace y código QR…"
      Quickshell.execDetached([binPath, "exec", actionId, targetFile])
    } else {
      root.dismiss()
      Quickshell.execDetached([binPath, "exec", actionId, targetFile])
    }
  }

  function sliceCenterAngleRad(i, count) {
    var step = (Math.PI * 2) / Math.max(1, count)
    return (-Math.PI / 2) + i * step
  }

  function updateHoverFromPoint(mx, my) {
    if (root.qrMode || root.slices.length === 0) return
    var cx = wheelContainer.width / 2
    var cy = wheelContainer.height / 2
    var dx = mx - cx
    var dy = my - cy
    var dist = Math.sqrt(dx * dx + dy * dy)

    if (dist < root.innerRadius - 8) {
      root.hoveredIndex = -1
      return
    }

    var count = root.slices.length
    var step = (Math.PI * 2) / count
    var angle = Math.atan2(dy, dx) - (-Math.PI / 2) + (step / 2)
    while (angle < 0) angle += Math.PI * 2
    while (angle >= Math.PI * 2) angle -= Math.PI * 2

    var idx = Math.floor(angle / step) % count
    if (idx !== root.hoveredIndex) {
      root.hoveredIndex = idx
    }
  }

  PanelWindow {
    id: panel
    visible: root.opened
    anchors { top: true; bottom: true; left: true; right: true }
    color: "transparent"
    WlrLayershell.namespace: "omarchy-pie"
    WlrLayershell.layer: WlrLayer.Overlay
    WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
    exclusionMode: ExclusionMode.Ignore

    Rectangle {
      anchors.fill: parent
      color: root.scrimColor
      opacity: 0.55
    }

    MouseArea {
      anchors.fill: parent
      hoverEnabled: true
      acceptedButtons: Qt.LeftButton | Qt.RightButton
      onPositionChanged: function(mouse) {
        var localPt = mapToItem(wheelContainer, mouse.x, mouse.y)
        root.updateHoverFromPoint(localPt.x, localPt.y)
      }
      onClicked: function(mouse) {
        if (mouse.button === Qt.RightButton) {
          if (!root.goBack()) root.dismiss()
          return
        }
        if (root.qrMode) {
          var qrPt = mapToItem(qrCard, mouse.x, mouse.y)
          if (qrPt.x < 0 || qrPt.y < 0 || qrPt.x > qrCard.width || qrPt.y > qrCard.height) {
            root.dismiss()
          }
          return
        }
        if (root.hoveredIndex >= 0) {
          root.triggerSlice(root.hoveredIndex)
        } else {
          var localPt = mapToItem(wheelContainer, mouse.x, mouse.y)
          var dx = localPt.x - wheelContainer.width / 2
          var dy = localPt.y - wheelContainer.height / 2
          var dist = Math.sqrt(dx * dx + dy * dy)
          if (dist < root.innerRadius && root.inSubMenu) {
            root.goBack()
          } else {
            root.dismiss()
          }
        }
      }
    }

    Item {
      id: keyCatcher
      anchors.fill: parent
      focus: true

      Keys.priority: Keys.BeforeItem
      Keys.onPressed: function(event) {
        if (event.key === Qt.Key_Escape) {
          if (!root.goBack()) root.dismiss()
          event.accepted = true
        } else if (event.key === Qt.Key_Backspace) {
          root.goBack()
          event.accepted = true
        } else if (event.key === Qt.Key_Q) {
          root.dismiss()
          event.accepted = true
        } else if (!root.qrMode && event.key >= Qt.Key_1 && event.key <= Qt.Key_8) {
          var idx = event.key - Qt.Key_1
          if (idx < root.slices.length) {
            root.hoveredIndex = idx
            root.triggerSlice(idx)
          }
          event.accepted = true
        } else if (!root.qrMode && (event.key === Qt.Key_Right || event.key === Qt.Key_Down || event.key === Qt.Key_Tab)) {
          if (root.slices.length > 0) {
            root.hoveredIndex = (root.hoveredIndex + 1 + root.slices.length) % root.slices.length
          }
          event.accepted = true
        } else if (!root.qrMode && (event.key === Qt.Key_Left || event.key === Qt.Key_Up)) {
          if (root.slices.length > 0) {
            root.hoveredIndex = (root.hoveredIndex <= 0 ? root.slices.length - 1 : root.hoveredIndex - 1)
          }
          event.accepted = true
        } else if (!root.qrMode && (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space)) {
          if (root.hoveredIndex >= 0) {
            root.triggerSlice(root.hoveredIndex)
          }
          event.accepted = true
        }
      }
    }

    // Radial Wheel Container positioned around cursor (clamped to screen margins)
    Item {
      id: wheelContainer
      width: root.wheelSize
      height: root.wheelSize
      visible: !root.qrMode
      scale: root.wheelScale

      Behavior on scale {
        NumberAnimation { duration: 150; easing.type: Easing.OutBack }
      }

      readonly property int margin: 24
      x: Math.max(margin, Math.min(panel.width - width - margin, root.cursorX - width / 2))
      y: Math.max(margin, Math.min(panel.height - height - margin, root.cursorY - height / 2))

      Canvas {
        id: pieCanvas
        anchors.fill: parent
        antialiasing: true

        onPaint: {
          var ctx = getContext("2d")
          ctx.clearRect(0, 0, width, height)

          var count = root.slices.length
          if (count === 0) return

          var cx = width / 2
          var cy = height / 2
          var step = (Math.PI * 2) / count
          var gapAngle = 0.042

          // Outer subtle ambient ring
          ctx.beginPath()
          ctx.arc(cx, cy, root.outerRadius + 10, 0, Math.PI * 2)
          ctx.strokeStyle = Qt.rgba(root.borderColor.r, root.borderColor.g, root.borderColor.b, 0.16)
          ctx.lineWidth = 1.5
          ctx.stroke()

          for (var i = 0; i < count; i++) {
            var isHovered = (i === root.hoveredIndex)
            var centerAng = (-Math.PI / 2) + i * step
            var startAng = centerAng - (step / 2) + (gapAngle / 2)
            var endAng = centerAng + (step / 2) - (gapAngle / 2)

            var rIn = isHovered ? root.innerRadius - 3 : root.innerRadius
            var rOut = isHovered ? root.outerRadius + 9 : root.outerRadius

            ctx.beginPath()
            ctx.arc(cx, cy, rOut, startAng, endAng, false)
            ctx.arc(cx, cy, rIn, endAng, startAng, true)
            ctx.closePath()

            if (isHovered) {
              ctx.fillStyle = Qt.rgba(root.accentColor.r, root.accentColor.g, root.accentColor.b, 0.24)
              ctx.strokeStyle = Qt.rgba(root.accentColor.r, root.accentColor.g, root.accentColor.b, 0.95)
              ctx.lineWidth = 2.4
            } else {
              ctx.fillStyle = Qt.rgba(root.bgColor.r, root.bgColor.g, root.bgColor.b, 0.92)
              ctx.strokeStyle = Qt.rgba(root.borderColor.r, root.borderColor.g, root.borderColor.b, 0.35)
              ctx.lineWidth = 1.4
            }

            ctx.fill()
            ctx.stroke()
          }
        }
      }

      // Slice Icons + Labels positioned at radial midpoints
      Repeater {
        model: root.slices.length

        delegate: Item {
          required property int index
          readonly property var sliceData: root.slices[index] || ({})
          readonly property bool isHovered: index === root.hoveredIndex
          readonly property real angle: root.sliceCenterAngleRad(index, root.slices.length)
          readonly property real midRadius: (root.innerRadius + root.outerRadius) / 2 + (isHovered ? 4 : 0)

          width: 105
          height: 68
          x: (wheelContainer.width / 2) + Math.cos(angle) * midRadius - width / 2
          y: (wheelContainer.height / 2) + Math.sin(angle) * midRadius - height / 2

          Behavior on x { NumberAnimation { duration: 90; easing.type: Easing.OutCubic } }
          Behavior on y { NumberAnimation { duration: 90; easing.type: Easing.OutCubic } }

          Column {
            anchors.centerIn: parent
            spacing: 3

            Text {
              anchors.horizontalCenter: parent.horizontalCenter
              text: sliceData.icon || "󰒓"
              font.family: root.fontFamily
              font.pixelSize: isHovered ? 26 : 22
              color: isHovered ? root.accentColor : root.fgColor
              horizontalAlignment: Text.AlignHCenter
            }

            Text {
              anchors.horizontalCenter: parent.horizontalCenter
              textFormat: Text.PlainText
              text: sliceData.label || ""
              font.family: root.fontFamily
              font.pixelSize: 12
              font.bold: isHovered
              color: isHovered ? root.fgColor : Qt.rgba(root.fgColor.r, root.fgColor.g, root.fgColor.b, 0.85)
              horizontalAlignment: Text.AlignHCenter
            }

            Rectangle {
              anchors.horizontalCenter: parent.horizontalCenter
              width: 18
              height: 15
              radius: 4
              color: isHovered
                ? Qt.rgba(root.accentColor.r, root.accentColor.g, root.accentColor.b, 0.3)
                : Qt.rgba(root.fgColor.r, root.fgColor.g, root.fgColor.b, 0.08)

              Text {
                anchors.centerIn: parent
                text: String(index + 1)
                font.family: root.fontFamily
                font.pixelSize: 9
                color: isHovered ? root.fgColor : root.mutedColor
              }
            }
          }
        }
      }

      // Center Hub (File Info + Hovered Action Description + Submenu Back Button)
      Rectangle {
        id: centerHub
        width: (root.innerRadius - 8) * 2
        height: (root.innerRadius - 8) * 2
        radius: width / 2
        anchors.centerIn: parent
        color: Qt.rgba(root.bgColor.r, root.bgColor.g, root.bgColor.b, 0.96)
        border.width: 2
        border.color: root.hoveredIndex >= 0 || root.inSubMenu
          ? root.accentColor
          : Qt.rgba(root.borderColor.r, root.borderColor.g, root.borderColor.b, 0.45)

        Column {
          anchors.centerIn: parent
          width: parent.width - 22
          spacing: 4

          Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: root.hoveredIndex >= 0 && root.slices[root.hoveredIndex]
              ? root.slices[root.hoveredIndex].icon
              : (root.inSubMenu ? "󰁍" : root.fileIcon)
            font.family: root.fontFamily
            font.pixelSize: 26
            color: root.accentColor
            horizontalAlignment: Text.AlignHCenter
          }

          Text {
            width: parent.width
            textFormat: Text.PlainText
            text: root.hoveredIndex >= 0 && root.slices[root.hoveredIndex]
              ? root.slices[root.hoveredIndex].label
              : (root.inSubMenu ? root.subMenuTitle : root.fileName)
            font.family: root.fontFamily
            font.pixelSize: 12
            font.bold: true
            color: root.fgColor
            elide: Text.ElideMiddle
            horizontalAlignment: Text.AlignHCenter
          }

          Text {
            width: parent.width
            textFormat: Text.PlainText
            text: root.hoveredIndex >= 0 && root.slices[root.hoveredIndex]
              ? root.slices[root.hoveredIndex].subtitle
              : (root.inSubMenu
                  ? "Click here or Esc to go back"
                  : (root.dimensions ? (root.dimensions + " • " + root.fileSize) : (root.fileKind + " • " + root.fileSize)))
            font.family: root.fontFamily
            font.pixelSize: 10
            color: root.mutedColor
            wrapMode: Text.WordWrap
            maximumLineCount: 2
            elide: Text.ElideRight
            horizontalAlignment: Text.AlignHCenter
          }
        }
      }
    }

    // QR Code / Instant Share Card
    Rectangle {
      id: qrCard
      visible: root.qrMode
      width: 310
      height: 360
      radius: Math.max(12, Style.cornerRadius)
      anchors.centerIn: parent
      color: Qt.rgba(root.bgColor.r, root.bgColor.g, root.bgColor.b, 0.97)
      border.width: 2
      border.color: root.accentColor

      Column {
        anchors.fill: parent
        anchors.margins: 20
        spacing: 12

        Text {
          width: parent.width
          textFormat: Text.PlainText
          text: root.fileName
          font.family: root.fontFamily
          font.pixelSize: 14
          font.bold: true
          color: root.fgColor
          elide: Text.ElideMiddle
          horizontalAlignment: Text.AlignHCenter
        }

        Rectangle {
          width: 210
          height: 210
          radius: 10
          anchors.horizontalCenter: parent.horizontalCenter
          color: "#ffffff"

          Image {
            anchors.fill: parent
            anchors.margins: 10
            visible: !root.qrLoading && root.qrSvgPath !== ""
            source: root.qrSvgPath ? ("file://" + root.qrSvgPath) : ""
            fillMode: Image.PreserveAspectFit
            cache: false
          }

          Text {
            anchors.centerIn: parent
            visible: root.qrLoading
            text: "󰐲\nGenerating QR…"
            font.family: root.fontFamily
            font.pixelSize: 14
            color: "#111318"
            horizontalAlignment: Text.AlignHCenter
          }
        }

        Text {
          width: parent.width
          textFormat: Text.PlainText
          text: root.qrSubtitle
          font.family: root.fontFamily
          font.pixelSize: 11
          color: root.accentColor
          horizontalAlignment: Text.AlignHCenter
          elide: Text.ElideRight
        }

        Text {
          width: parent.width
          textFormat: Text.PlainText
          text: root.qrUrl || "Listening for connections…"
          font.family: root.fontFamily
          font.pixelSize: 10
          color: root.mutedColor
          horizontalAlignment: Text.AlignHCenter
          elide: Text.ElideMiddle
        }

        Rectangle {
          width: 120
          height: 28
          radius: 6
          anchors.horizontalCenter: parent.horizontalCenter
          color: Qt.rgba(root.accentColor.r, root.accentColor.g, root.accentColor.b, 0.18)
          border.width: 1
          border.color: root.accentColor

          Text {
            anchors.centerIn: parent
            text: "Close (Esc)"
            font.family: root.fontFamily
            font.pixelSize: 11
            color: root.fgColor
          }

          MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: root.dismiss()
          }
        }
      }
    }
  }
}
