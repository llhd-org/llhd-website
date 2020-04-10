const SPEC_MARKDOWN_URL = "https://raw.githubusercontent.com/fabianschuiki/llhd/master/doc/LANGUAGE.md";

const md = require("markdown-it")({
		xhtmlOut: true,
		typographer: true,
	})
	.use(require("markdown-it-toc-and-anchor").default, {
		tocFirstLevel: 2,
		tocLastLevel: 4,
		tocClassName: null,
		anchorLinkSymbol: "§",
		anchorLinkSpace: false,
		anchorClassName: "markdown-anchor",
	});
const fs = require("fs");
const request = require("sync-request");

// Download the language spec.
var input = request("GET", SPEC_MARKDOWN_URL).getBody("utf8");

// Render the markdown.
let toc;
var output = md.render(input, {
	tocCallback: function(tocMarkdown, tocArray, tocHtml) {
		toc = tocHtml;
	}
});
toc = "";

// Write the spec file.
fs.writeFileSync("frontend/spec.html", `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no">
<link rel="stylesheet" href="https://stackpath.bootstrapcdn.com/bootstrap/4.3.1/css/bootstrap.min.css" integrity="sha384-ggOyR0iXCbMQv3Xipma34MD+dH/1fQ784/j6cY/iJTQUOhcWr7x9JvoRxT2MZw1T" crossorigin="anonymous">
<link href="https://fonts.googleapis.com/css?family=Big+Shoulders+Display:700|Open+Sans:700,400,400i&display=swap" rel="stylesheet">
<link href="spec.css" rel="stylesheet">
<title>LLHD Language Reference</title>
</head>
<body>
	<div class="container">
		<div class="markdown-body">
			<nav class="sidebar">
				${toc}
			</nav>
			${output}
		</div>
		<hr class="my-4">
		<footer>
			<p class="small text-center">
			Copyright © 2019 by <a href="https://github.com/fabianschuiki" target="blank">Fabian Schuiki</a>
			</p>
		</footer>
	</div>
</body>
</html>
`);
