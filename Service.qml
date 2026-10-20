import QtQuick
import Quickshell
import Quickshell.Io

Item {
  id: root

  property var shell: null
  property var manifest: null

  readonly property string homeDir: Quickshell.env("HOME")
  readonly property string sourceDir: {
    if (manifest && manifest.__sourceDir) return String(manifest.__sourceDir)
    var url = String(Qt.resolvedUrl("."))
    var p = url.indexOf("file://") === 0 ? url.substring(7) : url
    return p.replace(/\/$/, "")
  }

  Component.onCompleted: {
    // Auto-bootstrap ~/.local/bin/omarchy-pie symlink and Nautilus context script
    var setupCmd = "mkdir -p \"$HOME/.local/bin\" \"$HOME/.local/share/nautilus/scripts\" && "
      + "if [ -x \"" + root.sourceDir + "/bin/omarchy-pie\" ]; then "
      + "ln -sf \"" + root.sourceDir + "/bin/omarchy-pie\" \"$HOME/.local/bin/omarchy-pie\"; "
      + "fi && "
      + "printf '#!/usr/bin/env bash\\nfirst_file=\"$(printf \"%%s\\\\n\" \"$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS\" | head -n 1)\"\\nif [[ -n \"$first_file\" ]]; then exec \"$HOME/.local/bin/omarchy-pie\" \"$first_file\"; else exec \"$HOME/.local/bin/omarchy-pie\"; fi\\n' > \"$HOME/.local/share/nautilus/scripts/Omarchy Pie\" && "
      + "chmod +x \"$HOME/.local/share/nautilus/scripts/Omarchy Pie\""
    Quickshell.execDetached(["bash", "-c", setupCmd])
  }
}
