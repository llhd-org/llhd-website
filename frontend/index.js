var editor_hdl = ace.edit("editor-hdl");
var editor_llhd = ace.edit("editor-llhd");

editor_hdl.setOptions({
    showPrintMargin: false,
    theme: "ace/theme/tomorrow",
    mode: "ace/mode/verilog",
    wrap: true,
});

editor_llhd.setOptions({
    showPrintMargin: false,
    theme: "ace/theme/tomorrow",
    mode: "ace/mode/text",
    readOnly: true,
    wrap: true,
});

compileSpinner = $("div.compile-spin");
if (window.location.protocol)
    apiUrl = window.location.origin;
else
    apiUrl = "http://127.0.0.1:5000"

// Keep track of the code across page reloads.
if (!localStorage.code) {
    localStorage.code = "module Accumulator (\n\
    input  logic        clk,\n\
    input  logic        direction,\n\
    input  logic [15:0] increment,\n\
    output logic [15:0] result\n\
);\n\
    logic [15:0] next;\n\
\n\
    always_comb begin\n\
        if (direction)\n\
            next = result + increment;\n\
        else\n\
            next = result - increment;\n\
    end\n\
\n\
    always_ff @(posedge clk)\n\
        result <= next;\n\
endmodule";
}
editor_hdl.session.setValue(localStorage.code);
editor_hdl.session.on("change", function () {
    localStorage.code = editor_hdl.session.getValue();
    queueCompile();
});

// Compilation queuing
var compileTimeout

function queueCompile() {
    if (compileTimeout)
        clearTimeout(compileTimeout);
    compileTimeout = setTimeout(compile, 500);
}

function compile() {
    if (compileTimeout)
        clearTimeout(compileTimeout);
    compileSpinner.addClass("show");
    $.post(apiUrl + "/compile", JSON.stringify({
        code: editor_hdl.session.getValue(),
    }))
    .done(function(resp) {
        editor_llhd.session.setValue(resp.output);
    })
    .fail(function(resp) {
        editor_llhd.session.setValue("Error: " + resp.responseJSON.error);
    })
    .always(function() {
        compileSpinner.removeClass("show");
    });
}

compile();
