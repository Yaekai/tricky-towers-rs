// PreToolUse guard: game files (extracted/, research/, game/, the Steam install) never get
// committed or sent out. Blocks a tool call that names one of those paths together with a
// commit, upload or network command. Exit 2 = refuse, with the reason on stderr.
let input = "";
process.stdin.on("data", (d) => (input += d));
process.stdin.on("end", () => {
  let call;
  try {
    call = JSON.parse(input);
  } catch {
    process.exit(0);
  }
  const text = JSON.stringify(call.tool_input || {});
  const gameFiles = /(?<!docs[\\/]+)\b(extracted|research|game)[\\/]|steamapps|Tricky Towers/i;
  const sendsOut =
    /git\s+(add|commit|push|stash)|\bcurl\b|\bwget\b|Invoke-(WebRequest|RestMethod)|\bgh\s|\bscp\b|\brsync\b|upload|omniroute|https?:\/\//i;
  if (call.tool_name === "Bash" || call.tool_name === "PowerShell") {
    if (/git\s+add\s+(-f|--force)/i.test(text)) {
      console.error("Blocked: git add --force can pull ignored game files into the repo.");
      process.exit(2);
    }
    if (gameFiles.test(text) && sendsOut.test(text)) {
      console.error("Blocked: game files (extracted/, research/, game/, the install) stay local.");
      process.exit(2);
    }
  }
  if (call.tool_name === "Edit" || call.tool_name === "Write") {
    const path = String((call.tool_input || {}).file_path || "");
    if (/\.gitignore$/.test(path) && /!\s*\/?(extracted|research|game)/.test(text)) {
      console.error("Blocked: don't whitelist game-file folders in .gitignore.");
      process.exit(2);
    }
  }
  process.exit(0);
});
