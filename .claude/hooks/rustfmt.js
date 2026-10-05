// PostToolUse: format a Rust file right after it is edited.
const { spawnSync } = require("child_process");
let input = "";
process.stdin.on("data", (d) => (input += d));
process.stdin.on("end", () => {
  try {
    const path = JSON.parse(input).tool_input.file_path || "";
    if (path.endsWith(".rs")) {
      spawnSync("rustfmt", ["--edition", "2024", path], { stdio: "ignore", windowsHide: true });
    }
  } catch {}
  process.exit(0);
});
