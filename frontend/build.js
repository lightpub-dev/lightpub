const fse = require("fs-extra");

// copy files in public to dist
fse
  .mkdir("dist", {
    recursive: true,
  })
  .then(() => {
    fse.copy("public", "dist");
  });
