const fs = require("fs");
const path = require("path");

// Remove the auto-generated hook-0-api.info.mdx file
const introFile = path.join(__dirname, "../api/hook-0-api.info.mdx");
if (fs.existsSync(introFile)) {
  fs.unlinkSync(introFile);
  console.log("✅ Removed auto-generated hook-0-api.info.mdx");
}

// Update the sidebar to remove the hook-0-api reference
const sidebarFile = path.join(__dirname, "../api/sidebar.ts");
if (fs.existsSync(sidebarFile)) {
  let content = fs.readFileSync(sidebarFile, "utf8");
  
  // Remove the hook-0-api doc entry
  content = content.replace(/\s*{\s*type:\s*"doc",\s*id:\s*"api\/hook-0-api",?\s*},?\s*/g, "");
  
  fs.writeFileSync(sidebarFile, content);
  console.log("✅ Updated sidebar.ts to remove hook-0-api reference");
}

console.log("✅ Post-processing complete");
